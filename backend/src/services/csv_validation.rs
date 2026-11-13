use crate::models::transaction::{NewTransaction, TransactionType};
use crate::models::validation::Validate;
use crate::services::csv_normalization::CsvNormalizationReport;

/// A transaction that passed CSV-level domain validation.
#[derive(Debug)]
pub struct ValidatedCsvRecord {
    pub row_number: usize,
    pub transaction: NewTransaction,
}

/// A row that failed validation.
#[derive(Debug)]
pub struct CsvValidationRowError {
    pub row: usize,
    pub message: String,
}

/// Output of the validation pipeline.
#[derive(Debug)]
pub struct CsvValidationReport {
    pub total_rows: usize,
    pub records: Vec<ValidatedCsvRecord>,
    pub errors: Vec<CsvValidationRowError>,
}

/// Validate normalized CSV records without writing to the database.
pub fn validate_csv(normalized: CsvNormalizationReport) -> CsvValidationReport {
    let mut report = CsvValidationReport {
        total_rows: normalized.total_rows,
        records: Vec::new(),
        errors: normalized
            .errors
            .into_iter()
            .map(|error| CsvValidationRowError {
                row: error.row,
                message: error.message,
            })
            .collect(),
    };

    for normalized_record in normalized.records {
        let row_number = normalized_record.row_number;
        let record = normalized_record.record;

        let transaction_type = match TransactionType::parse(&record.transaction_type) {
            Some(kind) => kind,
            None => {
                report.errors.push(CsvValidationRowError {
                    row: row_number,
                    message: String::from("Unsupported transaction type"),
                });
                continue;
            }
        };

        let transaction = NewTransaction {
            external_reference: record.external_reference,
            amount_minor: record.amount_minor,
            currency: record.currency,
            sender_account: record.sender_account,
            recipient_account: record.recipient_account,
            transaction_type,
            occurred_at: record.occurred_at,
        };

        if let Err(message) = transaction.validate() {
            report.errors.push(CsvValidationRowError {
                row: row_number,
                message,
            });
            continue;
        }

        report.records.push(ValidatedCsvRecord {
            row_number,
            transaction,
        });
    }

    report.errors.sort_by_key(|error| error.row);

    report
}

#[cfg(test)]
mod tests {
    use super::validate_csv;
    use crate::services::csv_normalization::normalize_csv;
    use crate::services::csv_parser::parse_csv;

    const HEADERS: &str = "external_reference,amount_minor,currency,sender_account,recipient_account,transaction_type,occurred_at";

    fn validate_content(content: &str) -> super::CsvValidationReport {
        let parsed = parse_csv(content).unwrap();
        let normalized = normalize_csv(parsed);
        validate_csv(normalized)
    }

    fn valid_row(reference: &str) -> String {
        format!("{reference},5000,INR,ACC-100,ACC-200,transfer,2026-11-12T09:00:00Z")
    }

    #[test]
    fn accepts_valid_transaction() {
        let content = format!("{HEADERS}\n{}", valid_row("TXN-001"));

        let report = validate_content(&content);

        assert_eq!(report.total_rows, 1);
        assert_eq!(report.records.len(), 1);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn rejects_zero_and_negative_amounts() {
        let content = format!(
            "{HEADERS}\nTXN-001,0,INR,ACC-100,ACC-200,transfer,2026-11-12T09:00:00Z\nTXN-002,-500,INR,ACC-100,ACC-200,transfer,2026-11-12T09:00:00Z"
        );

        let report = validate_content(&content);

        assert!(report.records.is_empty());
        assert_eq!(report.errors.len(), 2);
        assert_eq!(report.errors[0].row, 2);
        assert_eq!(report.errors[1].row, 3);
    }

    #[test]
    fn rejects_unsupported_transaction_type() {
        let content = format!(
            "{HEADERS}\nTXN-001,5000,INR,ACC-100,ACC-200,crypto_transfer,2026-11-12T09:00:00Z"
        );

        let report = validate_content(&content);

        assert!(report.records.is_empty());
        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.errors[0].message, "Unsupported transaction type");
    }

    #[test]
    fn keeps_valid_rows_when_another_row_fails_validation() {
        let content = format!(
            "{HEADERS}\n{}\nTXN-002,-100,INR,ACC-100,ACC-200,transfer,2026-11-12T09:00:00Z\n{}",
            valid_row("TXN-001"),
            valid_row("TXN-003")
        );

        let report = validate_content(&content);

        assert_eq!(report.total_rows, 3);
        assert_eq!(report.records.len(), 2);
        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.errors[0].row, 3);
    }

    #[test]
    fn combines_parser_errors_with_validation_errors_in_row_order() {
        let content = format!(
            "{HEADERS}\nTXN-001,hello,INR,ACC-100,ACC-200,transfer,2026-11-12T09:00:00Z\nTXN-002,-100,INR,ACC-100,ACC-200,transfer,2026-11-12T09:00:00Z"
        );

        let report = validate_content(&content);

        assert_eq!(report.errors.len(), 2);
        assert_eq!(report.errors[0].row, 2);
        assert_eq!(report.errors[1].row, 3);
    }
}
