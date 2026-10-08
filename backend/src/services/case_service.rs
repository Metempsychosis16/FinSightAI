use crate::models::case_response::CaseRow;
use crate::models::status::CaseStatus;
use crate::repositories::case_repository::CaseRepository;
use crate::repositories::error::RepositoryError;
use sqlx::PgPool;

#[derive(Debug)]
pub enum CaseServiceError {
    Repository(RepositoryError),
    NotFound,
    InvalidStoredStatus,
    InvalidTransition,
}

pub struct CaseService<'a> {
    repository: CaseRepository<'a>,
}

impl<'a> CaseService<'a> {
    pub fn new(db: &'a PgPool) -> Self {
        Self {
            repository: CaseRepository::new(db),
        }
    }

    pub async fn update_status(
        &self,
        case_id: i64,
        next_status: CaseStatus,
    ) -> Result<CaseRow, CaseServiceError> {
        let current_status = self
            .repository
            .find_status(case_id)
            .await
            .map_err(CaseServiceError::Repository)?;

        let current_status = match current_status {
            Some(status) => {
                CaseStatus::from_str(&status).map_err(|_| CaseServiceError::InvalidStoredStatus)?
            }

            None => return Err(CaseServiceError::NotFound),
        };

        current_status
            .transition_to(next_status)
            .map_err(|_| CaseServiceError::InvalidTransition)?;

        let row = self
            .repository
            .update_status(case_id, next_status.as_str())
            .await
            .map_err(CaseServiceError::Repository)?;

        Ok(row)
    }
}
