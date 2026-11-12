use chrono::Utc;
use sqlx::PgPool;

use crate::models::evidence::{Evidence, EvidencePackage, NewEvidence};
use crate::repositories::evidence_repository::{EvidenceRepository, EvidenceRepositoryError};

#[derive(Debug)]
pub enum EvidenceServiceError {
    InvalidInput(String),
    CaseNotFound,
    Repository(EvidenceRepositoryError),
}

pub struct EvidenceService<'a> {
    repository: EvidenceRepository<'a>,
}

impl<'a> EvidenceService<'a> {
    pub fn new(db: &'a PgPool) -> Self {
        Self {
            repository: EvidenceRepository::new(db),
        }
    }

    pub async fn create(&self, evidence: NewEvidence) -> Result<Evidence, EvidenceServiceError> {
        let evidence = evidence.normalize();

        evidence
            .validate()
            .map_err(EvidenceServiceError::InvalidInput)?;

        self.repository
            .create(&evidence)
            .await
            .map_err(EvidenceServiceError::Repository)
    }

    pub async fn create_package(
        &self,
        case_id: i64,
    ) -> Result<EvidencePackage, EvidenceServiceError> {
        if case_id <= 0 {
            return Err(EvidenceServiceError::InvalidInput(String::from(
                "Case ID must be greater than zero",
            )));
        }

        let evidence =
            self.repository
                .list_by_case(case_id)
                .await
                .map_err(|error| match error {
                    EvidenceRepositoryError::CaseNotFound => EvidenceServiceError::CaseNotFound,
                    other => EvidenceServiceError::Repository(other),
                })?;

        let evidence_count = evidence.len();

        Ok(EvidencePackage {
            case_id,
            generated_at: Utc::now(),
            evidence_count,
            evidence,
        })
    }
}
