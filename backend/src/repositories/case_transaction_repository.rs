use sqlx::PgPool;

#[derive(Debug)]
pub enum CaseTransactionRepositoryError {
    Database(sqlx::Error),
    CaseNotFound,
    TransactionNotFound,
    AlreadyLinked,
}

pub struct CaseTransactionRepository<'a> {
    db: &'a PgPool,
}

impl<'a> CaseTransactionRepository<'a> {
    pub fn new(db: &'a PgPool) -> Self {
        Self { db }
    }

    pub async fn link(
        &self,
        case_id: i64,
        transaction_id: i64,
    ) -> Result<(), CaseTransactionRepositoryError> {
        let case_exists =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM cases WHERE id = $1)")
                .bind(case_id)
                .fetch_one(self.db)
                .await
                .map_err(CaseTransactionRepositoryError::Database)?;

        if !case_exists {
            return Err(CaseTransactionRepositoryError::CaseNotFound);
        }

        let transaction_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM transactions WHERE id = $1)",
        )
        .bind(transaction_id)
        .fetch_one(self.db)
        .await
        .map_err(CaseTransactionRepositoryError::Database)?;

        if !transaction_exists {
            return Err(CaseTransactionRepositoryError::TransactionNotFound);
        }

        let result = sqlx::query(
            r#"
            INSERT INTO case_transactions (case_id, transaction_id)
            VALUES ($1, $2)
            ON CONFLICT (case_id, transaction_id) DO NOTHING
            "#,
        )
        .bind(case_id)
        .bind(transaction_id)
        .execute(self.db)
        .await
        .map_err(CaseTransactionRepositoryError::Database)?;

        if result.rows_affected() == 0 {
            return Err(CaseTransactionRepositoryError::AlreadyLinked);
        }

        Ok(())
    }
}
