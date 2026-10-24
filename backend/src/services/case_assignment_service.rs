use crate::repositories::case_assignment_repository::CaseAssignmentRepository;
use sqlx::PgPool;

#[derive(Debug)]
pub enum CaseAssignmentServiceError {
    Database(sqlx::Error),
    CaseNotFound,
    InvestigatorNotAssignable,
}

pub struct CaseAssignmentService<'a> {
    repository: CaseAssignmentRepository<'a>,
}

impl<'a> CaseAssignmentService<'a> {
    pub fn new(db: &'a PgPool) -> Self {
        Self {
            repository: CaseAssignmentRepository::new(db),
        }
    }

    pub async fn assign(
        &self,
        case_id: i64,
        investigator_id: i64,
    ) -> Result<(), CaseAssignmentServiceError> {
        if case_id <= 0 || investigator_id <= 0 {
            return Err(CaseAssignmentServiceError::CaseNotFound);
        }

        let assignable = self
            .repository
            .investigator_is_assignable(investigator_id)
            .await
            .map_err(CaseAssignmentServiceError::Database)?;

        if !assignable {
            return Err(CaseAssignmentServiceError::InvestigatorNotAssignable);
        }

        let updated = self
            .repository
            .assign(case_id, investigator_id)
            .await
            .map_err(CaseAssignmentServiceError::Database)?;

        if !updated {
            return Err(CaseAssignmentServiceError::CaseNotFound);
        }

        Ok(())
    }
}
