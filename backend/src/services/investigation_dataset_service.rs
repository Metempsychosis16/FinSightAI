use sqlx::PgPool;

use crate::models::case_response::{CaseResponse, CaseResponseError};
use crate::models::investigation_dataset::{DatasetInvestigator, InvestigationDataset};
use crate::repositories::investigation_dataset_repository::{
    InvestigationDatasetRepository, InvestigationDatasetRows,
};

#[derive(Debug)]
pub enum InvestigationDatasetError {
    InvalidId,
    NotFound,
    InvalidStoredStatus,
    Database(sqlx::Error),
}

pub struct InvestigationDatasetService<'a> {
    repository: InvestigationDatasetRepository<'a>,
}

// Validate before attempting database access.
fn validate_case_id(case_id: i64) -> Result<(), InvestigationDatasetError> {
    if case_id <= 0 {
        return Err(InvestigationDatasetError::InvalidId);
    }

    Ok(())
}

impl<'a> InvestigationDatasetService<'a> {
    pub fn new(db: &'a PgPool) -> Self {
        Self {
            repository: InvestigationDatasetRepository::new(db),
        }
    }

    pub async fn build(
        &self,
        case_id: i64,
    ) -> Result<InvestigationDataset, InvestigationDatasetError> {
        validate_case_id(case_id)?;

        let rows = self
            .repository
            .load(case_id)
            .await
            .map_err(InvestigationDatasetError::Database)?
            .ok_or(InvestigationDatasetError::NotFound)?;

        Self::assemble(rows)
    }

    fn assemble(
        rows: InvestigationDatasetRows,
    ) -> Result<InvestigationDataset, InvestigationDatasetError> {
        let case = CaseResponse::try_from(rows.case).map_err(|error| match error {
            CaseResponseError::InvalidStatus => InvestigationDatasetError::InvalidStoredStatus,
        })?;

        let investigator = rows.investigator.map(|investigator| DatasetInvestigator {
            id: investigator.id,
            user_id: investigator.user_id,
            name: investigator.name,
            department: investigator.department,
            years_experience: investigator.years_experience,
        });

        let transaction_count = rows.transactions.len();
        let evidence_count = rows.evidence.len();

        Ok(InvestigationDataset {
            case,
            investigator,
            transaction_count,
            transactions: rows.transactions,
            evidence_count,
            evidence: rows.evidence,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{InvestigationDatasetError, validate_case_id};

    #[test]
    fn rejects_nonpositive_case_ids() {
        assert!(matches!(
            validate_case_id(0),
            Err(InvestigationDatasetError::InvalidId)
        ));

        assert!(matches!(
            validate_case_id(-42),
            Err(InvestigationDatasetError::InvalidId)
        ));
    }

    #[test]
    fn accepts_positive_case_ids() {
        assert!(validate_case_id(1).is_ok());
        assert!(validate_case_id(42).is_ok());
    }
}
