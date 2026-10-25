use crate::models::transaction::{NewTransaction, Transaction, TransactionType};
use crate::models::validation::Validate;
use crate::repositories::transaction_repository::{
    TransactionRepository, TransactionRepositoryError,
};
use sqlx::PgPool;

#[derive(Debug)]
pub enum TransactionServiceError {
    InvalidInput(String),
    DuplicateReference,
    Repository(TransactionRepositoryError),
}

pub struct TransactionService<'a> {
    repository: TransactionRepository<'a>,
}

impl<'a> TransactionService<'a> {
    pub fn new(db: &'a PgPool) -> Self {
        Self {
            repository: TransactionRepository::new(db),
        }
    }

    pub async fn ingest(
        &self,
        external_reference: String,
        amount_minor: i64,
        currency: String,
        sender_account: String,
        recipient_account: String,
        transaction_type: &str,
        occurred_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<Transaction, TransactionServiceError> {
        let transaction_type = TransactionType::parse(transaction_type).ok_or_else(|| {
            TransactionServiceError::InvalidInput(String::from("Unsupported transaction type"))
        })?;

        let transaction = NewTransaction {
            external_reference,
            amount_minor,
            currency,
            sender_account,
            recipient_account,
            transaction_type,
            occurred_at,
        }
        .normalize();

        transaction
            .validate()
            .map_err(TransactionServiceError::InvalidInput)?;

        match self.repository.create(&transaction).await {
            Ok(saved) => Ok(saved),
            Err(TransactionRepositoryError::Database(error)) => {
                if let sqlx::Error::Database(database_error) = &error {
                    if database_error.code().as_deref() == Some("23505") {
                        return Err(TransactionServiceError::DuplicateReference);
                    }
                }

                Err(TransactionServiceError::Repository(
                    TransactionRepositoryError::Database(error),
                ))
            }
            Err(error) => Err(TransactionServiceError::Repository(error)),
        }
    }
}
