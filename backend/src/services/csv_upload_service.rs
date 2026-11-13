use sqlx::PgPool;

use crate::services::bulk_transaction_service::{BulkImportError, BulkImportReport, import_csv};

pub const MAX_UPLOAD_BYTES: usize = 1_000_000;

#[derive(Debug)]
pub enum CsvUploadError {
    FileTooLarge,
    EmptyFile,
    Processing(BulkImportError),
}

pub struct CsvUploadService<'a> {
    db: &'a PgPool,
}

impl<'a> CsvUploadService<'a> {
    pub fn new(db: &'a PgPool) -> Self {
        Self { db }
    }

    pub async fn process(&self, content: &str) -> Result<BulkImportReport, CsvUploadError> {
        validate_upload(content)?;

        import_csv(self.db, content)
            .await
            .map_err(CsvUploadError::Processing)
    }
}

fn validate_upload(content: &str) -> Result<(), CsvUploadError> {
    if content.len() > MAX_UPLOAD_BYTES {
        return Err(CsvUploadError::FileTooLarge);
    }

    if content.trim().is_empty() {
        return Err(CsvUploadError::EmptyFile);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{CsvUploadError, MAX_UPLOAD_BYTES, validate_upload};

    #[test]
    fn accepts_nonempty_csv_content() {
        let csv = "external_reference,amount_minor\nTXN-001,5000";

        assert!(validate_upload(csv).is_ok());
    }

    #[test]
    fn rejects_empty_content() {
        assert!(matches!(
            validate_upload("   "),
            Err(CsvUploadError::EmptyFile)
        ));
    }

    #[test]
    fn rejects_oversized_content() {
        let content = "x".repeat(MAX_UPLOAD_BYTES + 1);

        assert!(matches!(
            validate_upload(&content),
            Err(CsvUploadError::FileTooLarge)
        ));
    }
}
