use chrono::{DateTime, Utc};
use serde::Deserialize;

const MAX_CSV_ROWS: usize = 500;

const REQUIRED_HEADERS: [&str; 7] = [
    "external_reference",
    "amount_minor",
    "currency",
    "sender_account",
    "recipient_account",
    "transaction_type",
    "occurred_at",
];

/// One CSV row after successful deserialization.
#[derive(Debug, Deserialize)]
pub struct CsvTransactionRecord {
    pub external_reference: String,
    pub amount_minor: i64,
    pub currency: String,
    pub sender_account: String,
    pub recipient_account: String,
    pub transaction_type: String,
    pub occurred_at: DateTime<Utc>,
}

/// A successfully parsed row and its original CSV row number.
#[derive(Debug)]
pub struct ParsedCsvRecord {
    pub row_number: usize,
    pub record: CsvTransactionRecord,
}

/// A row that could not be deserialized.
#[derive(Debug)]
pub struct CsvParseRowError {
    pub row: usize,
    pub message: String,
}

/// Results from parsing a complete CSV file.
#[derive(Debug)]
pub struct CsvParseReport {
    pub total_rows: usize,
    pub records: Vec<ParsedCsvRecord>,
    pub errors: Vec<CsvParseRowError>,
}

#[derive(Debug)]
pub enum CsvParseError {
    InvalidCsv(String),
    MissingHeaders(Vec<String>),
    TooManyRows,
}

pub fn parse_csv(content: &str) -> Result<CsvParseReport, CsvParseError> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(content.as_bytes());

    let headers = reader
        .headers()
        .map_err(|error| CsvParseError::InvalidCsv(error.to_string()))?
        .clone();

    let missing: Vec<String> = REQUIRED_HEADERS
        .iter()
        .filter(|required| !headers.iter().any(|header| header == **required))
        .map(|header| (*header).to_owned())
        .collect();

    if !missing.is_empty() {
        return Err(CsvParseError::MissingHeaders(missing));
    }

    let mut report = CsvParseReport {
        total_rows: 0,
        records: Vec::new(),
        errors: Vec::new(),
    };

    for (index, result) in reader.deserialize::<CsvTransactionRecord>().enumerate() {
        let row_number = index + 2;
        report.total_rows += 1;

        if report.total_rows > MAX_CSV_ROWS {
            return Err(CsvParseError::TooManyRows);
        }

        match result {
            Ok(record) => {
                report.records.push(ParsedCsvRecord { row_number, record });
            }

            Err(error) => {
                report.errors.push(CsvParseRowError {
                    row: row_number,
                    message: format!("Invalid CSV row: {error}"),
                });
            }
        }
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::{CsvParseError, parse_csv};

    const VALID_HEADERS: &str = "external_reference,amount_minor,currency,sender_account,recipient_account,transaction_type,occurred_at";

    fn valid_row(reference: &str) -> String {
        format!("{reference},5000,INR,ACC-100,ACC-200,transfer,2026-11-12T09:00:00Z")
    }

    #[test]
    fn parses_a_valid_transaction_row() {
        let content = format!("{VALID_HEADERS}\n{}", valid_row("TXN-001"));

        let report = parse_csv(&content).unwrap();

        assert_eq!(report.total_rows, 1);
        assert_eq!(report.records.len(), 1);
        assert_eq!(report.errors.len(), 0);

        let record = &report.records[0].record;

        assert_eq!(record.external_reference, "TXN-001");
        assert_eq!(record.amount_minor, 5000);
        assert_eq!(record.currency, "INR");
    }

    #[test]
    fn rejects_missing_headers() {
        let content = "external_reference,amount_minor\nTXN-001,5000";

        let result = parse_csv(content);

        assert!(matches!(result, Err(CsvParseError::MissingHeaders(_))));
    }

    #[test]
    fn records_invalid_row_without_discarding_valid_rows() {
        let content = format!(
            "{VALID_HEADERS}\n{}\nTXN-002,hello,INR,ACC-200,ACC-300,payment,2026-11-12T09:10:00Z\n{}",
            valid_row("TXN-001"),
            valid_row("TXN-003")
        );

        let report = parse_csv(&content).unwrap();

        assert_eq!(report.total_rows, 3);
        assert_eq!(report.records.len(), 2);
        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.errors[0].row, 3);
    }

    #[test]
    fn preserves_original_csv_row_numbers() {
        let content = format!(
            "{VALID_HEADERS}\n{}\nTXN-002,hello,INR,ACC-200,ACC-300,payment,2026-11-12T09:10:00Z",
            valid_row("TXN-001")
        );

        let report = parse_csv(&content).unwrap();

        assert_eq!(report.records[0].row_number, 2);
        assert_eq!(report.errors[0].row, 3);
    }

    #[test]
    fn rejects_more_than_500_rows() {
        let mut content = format!("{VALID_HEADERS}\n");

        for index in 0..501 {
            content.push_str(&valid_row(&format!("TXN-{index:03}")));
            content.push('\n');
        }

        assert!(matches!(
            parse_csv(&content),
            Err(CsvParseError::TooManyRows)
        ));
    }
}
