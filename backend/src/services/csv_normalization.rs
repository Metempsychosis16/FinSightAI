use crate::services::csv_parser::{CsvParseReport, CsvTransactionRecord};

/// One CSV record after normalization.
#[derive(Debug)]
pub struct NormalizedCsvRecord {
    pub row_number: usize,
    pub record: CsvTransactionRecord,
}

/// Output of the normalization stage.
#[derive(Debug)]
pub struct CsvNormalizationReport {
    pub total_rows: usize,
    pub records: Vec<NormalizedCsvRecord>,
    pub errors: Vec<CsvNormalizationRowError>,
}

/// Errors already discovered by the parser.
#[derive(Debug)]
pub struct CsvNormalizationRowError {
    pub row: usize,
    pub message: String,
}

/// Normalize fields with well-defined canonical representations.
pub fn normalize_csv(parsed: CsvParseReport) -> CsvNormalizationReport {
    let mut report = CsvNormalizationReport {
        total_rows: parsed.total_rows,
        records: Vec::new(),
        errors: parsed
            .errors
            .into_iter()
            .map(|error| CsvNormalizationRowError {
                row: error.row,
                message: error.message,
            })
            .collect(),
    };

    for parsed_record in parsed.records {
        let row_number = parsed_record.row_number;
        let mut record = parsed_record.record;

        record.external_reference = record.external_reference.trim().to_owned();

        record.currency = record.currency.trim().to_ascii_uppercase();

        record.sender_account = record.sender_account.trim().to_owned();

        record.recipient_account = record.recipient_account.trim().to_owned();

        record.transaction_type = record.transaction_type.trim().to_ascii_lowercase();

        report
            .records
            .push(NormalizedCsvRecord { row_number, record });
    }

    report
}

#[cfg(test)]
mod tests {
    use super::normalize_csv;
    use crate::services::csv_parser::parse_csv;

    const HEADERS: &str = "external_reference,amount_minor,currency,sender_account,recipient_account,transaction_type,occurred_at";

    #[test]
    fn trims_reference_and_account_whitespace() {
        let content = format!(
            "{HEADERS}\n  TXN-001  ,5000,INR,  Account-A  , Account-B ,transfer,2026-11-12T09:00:00Z"
        );

        let report = normalize_csv(parse_csv(&content).unwrap());

        let record = &report.records[0].record;

        assert_eq!(record.external_reference, "TXN-001");
        assert_eq!(record.sender_account, "Account-A");
        assert_eq!(record.recipient_account, "Account-B");
    }

    #[test]
    fn converts_currency_to_uppercase() {
        let content =
            format!("{HEADERS}\nTXN-001,5000, inr ,ACC-100,ACC-200,transfer,2026-11-12T09:00:00Z");

        let report = normalize_csv(parse_csv(&content).unwrap());

        assert_eq!(report.records[0].record.currency, "INR");
    }

    #[test]
    fn converts_transaction_type_to_lowercase() {
        let content =
            format!("{HEADERS}\nTXN-001,5000,INR,ACC-100,ACC-200, TRANSFER ,2026-11-12T09:00:00Z");

        let report = normalize_csv(parse_csv(&content).unwrap());

        assert_eq!(report.records[0].record.transaction_type, "transfer");
    }

    #[test]
    fn preserves_case_of_account_identifiers_and_references() {
        let content = format!(
            "{HEADERS}\nTxn-AbC-001,5000,INR,Account-AbC,Account-XyZ,transfer,2026-11-12T09:00:00Z"
        );

        let report = normalize_csv(parse_csv(&content).unwrap());

        let record = &report.records[0].record;

        assert_eq!(record.external_reference, "Txn-AbC-001");
        assert_eq!(record.sender_account, "Account-AbC");
        assert_eq!(record.recipient_account, "Account-XyZ");
    }

    #[test]
    fn preserves_amount_minor_without_rounding() {
        let content =
            format!("{HEADERS}\nTXN-001,125050,INR,ACC-100,ACC-200,transfer,2026-11-12T09:00:00Z");

        let report = normalize_csv(parse_csv(&content).unwrap());

        assert_eq!(report.records[0].record.amount_minor, 125050);
    }

    #[test]
    fn preserves_parser_errors_and_row_numbers() {
        let content =
            format!("{HEADERS}\nTXN-001,hello,INR,ACC-100,ACC-200,transfer,2026-11-12T09:00:00Z");

        let report = normalize_csv(parse_csv(&content).unwrap());

        assert_eq!(report.total_rows, 1);
        assert!(report.records.is_empty());
        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.errors[0].row, 2);
    }
}
