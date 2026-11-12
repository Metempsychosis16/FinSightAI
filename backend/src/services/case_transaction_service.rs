use sqlx::PgPool;

use crate::repositories::case_transaction_repository::{
    CaseTransactionRepository, CaseTransactionRepositoryError,
};

#[derive(Debug)]
pub enum CaseTransactionServiceError {
    InvalidId,
    Repository(CaseTransactionRepositoryError),
}

pub struct CaseTransactionService<'a> {
    repository: CaseTransactionRepository<'a>,
}

impl<'a> CaseTransactionService<'a> {
    pub fn new(db: &'a PgPool) -> Self {
        Self {
            repository: CaseTransactionRepository::new(db),
        }
    }

    pub async fn link(
        &self,
        case_id: i64,
        transaction_id: i64,
    ) -> Result<(), CaseTransactionServiceError> {
        if case_id <= 0 || transaction_id <= 0 {
            return Err(CaseTransactionServiceError::InvalidId);
        }

        self.repository
            .link(case_id, transaction_id)
            .await
            .map_err(CaseTransactionServiceError::Repository)
    }
}
