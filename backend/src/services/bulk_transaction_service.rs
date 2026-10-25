use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::PgPool;

use crate::models::transaction::Transaction;
use crate::services::transaction_service::{TransactionService, TransactionServiceError};

const MAX_CSV_BYTES: usize = 1_000_000;
const MAX_CSV_ROWS: usize = 500;

#[derive(Debug, Deserialize)]
struct CsvTransactionRecord {
    external_reference: String,
    amount_minor: i64,
    currency: String,
    sender_account: String,
    recipient_account: String,
    transaction_type: String,
    occurred_at: DateTime<Utc>,
}

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

    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(content.as_bytes());

    let headers = reader
        .headers()
        .map_err(|error| BulkImportError::InvalidCsv(error.to_string()))?
        .clone();

    let required = [
        "external_reference",
        "amount_minor",
        "currency",
        "sender_account",
        "recipient_account",
        "transaction_type",
        "occurred_at",
    ];

    let missing: Vec<String> = required
        .iter()
        .filter(|required_header| !headers.iter().any(|h| h == **required_header))
        .map(|header| (*header).to_owned())
        .collect();

    if !missing.is_empty() {
        return Err(BulkImportError::MissingHeaders(missing));
    }

    let mut report = BulkImportReport {
        total_rows: 0,
        imported: 0,
        failed: 0,
        errors: Vec::new(),
        transactions: Vec::new(),
    };

    let service = TransactionService::new(db);

    for (index, result) in reader.deserialize::<CsvTransactionRecord>().enumerate() {
        let row_number = index + 2;
        report.total_rows += 1;

        if report.total_rows > MAX_CSV_ROWS {
            return Err(BulkImportError::TooManyRows);
        }

        let record = match result {
            Ok(record) => record,
            Err(error) => {
                report.failed += 1;
                report.errors.push(BulkImportRowError {
                    row: row_number,
                    message: format!("Invalid CSV row: {error}"),
                });
                continue;
            }
        };

        let result = service
            .ingest(
                record.external_reference,
                record.amount_minor,
                record.currency,
                record.sender_account,
                record.recipient_account,
                &record.transaction_type,
                record.occurred_at,
            )
            .await;

        match result {
            Ok(transaction) => {
                report.imported += 1;
                report.transactions.push(TransactionSummary {
                    id: transaction.id,
                    external_reference: transaction.external_reference,
                });
            }
            Err(TransactionServiceError::InvalidInput(message)) => {
                report.failed += 1;
                report.errors.push(BulkImportRowError {
                    row: row_number,
                    message,
                });
            }
            Err(TransactionServiceError::DuplicateReference) => {
                report.failed += 1;
                report.errors.push(BulkImportRowError {
                    row: row_number,
                    message: String::from("External reference already exists"),
                });
            }
            Err(TransactionServiceError::Repository(_)) => {
                report.failed += 1;
                report.errors.push(BulkImportRowError {
                    row: row_number,
                    message: String::from("Database error while importing this row"),
                });
            }
        }
    }

    Ok(report)
}
