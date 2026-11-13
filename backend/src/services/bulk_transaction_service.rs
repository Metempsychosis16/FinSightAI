use std::collections::{HashMap, HashSet};

use sqlx::PgPool;

use crate::repositories::transaction_repository::TransactionRepository;
use crate::services::csv_normalization::normalize_csv;
use crate::services::csv_parser::{CsvParseError, parse_csv};
use crate::services::csv_validation::validate_csv;

const MAX_CSV_BYTES: usize = 1_000_000;

#[derive(Debug, serde::Serialize)]
pub struct BulkImportRowError {
    pub row: usize,
    pub message: String,
}

#[derive(Debug, serde::Serialize)]
pub struct BulkImportReport {
    pub total_rows: usize,
    pub imported: usize,
    pub failed: usize,
    pub errors: Vec<BulkImportRowError>,
    pub transactions: Vec<TransactionSummary>,
}

#[derive(Debug, serde::Serialize)]
pub struct TransactionSummary {
    pub id: i64,
    pub external_reference: String,
}

#[derive(Debug)]
pub enum BulkImportError {
    FileTooLarge,
    TooManyRows,
    InvalidCsv(String),
    MissingHeaders(Vec<String>),
}

pub async fn import_csv(db: &PgPool, content: &str) -> Result<BulkImportReport, BulkImportError> {
    if content.len() > MAX_CSV_BYTES {
        return Err(BulkImportError::FileTooLarge);
    }

    // Stage 1: Parse CSV text.
    let parsed = parse_csv(content).map_err(|error| match error {
        CsvParseError::InvalidCsv(message) => BulkImportError::InvalidCsv(message),
        CsvParseError::MissingHeaders(headers) => BulkImportError::MissingHeaders(headers),
        CsvParseError::TooManyRows => BulkImportError::TooManyRows,
    })?;

    // Stage 2: Normalize financial data.
    let normalized = normalize_csv(parsed);

    // Stage 3: Validate normalized records.
    let validated = validate_csv(normalized);

    let mut report = BulkImportReport {
        total_rows: validated.total_rows,
        imported: 0,
        failed: validated.errors.len(),
        errors: validated
            .errors
            .into_iter()
            .map(|error| BulkImportRowError {
                row: error.row,
                message: error.message,
            })
            .collect(),
        transactions: Vec::new(),
    };

    // Stage 4: Remove duplicate references within this CSV.
    let mut seen_references = HashSet::new();
    let mut row_by_reference = HashMap::new();
    let mut transactions_to_insert = Vec::new();

    for validated_record in validated.records {
        let row_number = validated_record.row_number;
        let transaction = validated_record.transaction;
        let reference = transaction.external_reference.clone();

        if !seen_references.insert(reference.clone()) {
            report.failed += 1;
            report.errors.push(BulkImportRowError {
                row: row_number,
                message: String::from("External reference appears more than once in this CSV"),
            });
            continue;
        }

        row_by_reference.insert(reference, row_number);
        transactions_to_insert.push(transaction);
    }

    // Stage 5: Insert all unique, validated transactions in one query.
    let repository = TransactionRepository::new(db);

    match repository.create_many(&transactions_to_insert).await {
        Ok(inserted) => {
            let mut inserted_references = HashSet::new();

            for transaction in inserted {
                inserted_references.insert(transaction.external_reference.clone());

                report.transactions.push(TransactionSummary {
                    id: transaction.id,
                    external_reference: transaction.external_reference,
                });
            }

            // References not returned by PostgreSQL conflicted with
            // existing records in the database.
            for (reference, row_number) in row_by_reference {
                if !inserted_references.contains(&reference) {
                    report.failed += 1;

                    report.errors.push(BulkImportRowError {
                        row: row_number,
                        message: String::from("External reference already exists"),
                    });
                }
            }

            report.imported = report.transactions.len();
        }

        Err(_) => {
            // The bulk statement failed, so none of its rows were
            // inserted by that statement.
            for (reference, row_number) in row_by_reference {
                report.failed += 1;

                report.errors.push(BulkImportRowError {
                    row: row_number,
                    message: format!("Database error while importing transaction {reference}"),
                });
            }
        }
    }

    report.errors.sort_by_key(|error| error.row);

    Ok(report)
}
