use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::models::transaction::{NewTransaction, Transaction, TransactionType};

#[derive(Debug)]
pub enum TransactionRepositoryError {
    Database(sqlx::Error),
    InvalidStoredTransactionType(String),
}

#[derive(Debug, sqlx::FromRow)]
struct TransactionRow {
    id: i64,
    external_reference: String,
    amount_minor: i64,
    currency: String,
    sender_account: String,
    recipient_account: String,
    transaction_type: String,
    occurred_at: DateTime<Utc>,
}

impl TryFrom<TransactionRow> for Transaction {
    type Error = TransactionRepositoryError;

    fn try_from(row: TransactionRow) -> Result<Self, Self::Error> {
        let transaction_type = TransactionType::parse(&row.transaction_type).ok_or_else(|| {
            TransactionRepositoryError::InvalidStoredTransactionType(row.transaction_type.clone())
        })?;

        Ok(Self {
            id: row.id,
            external_reference: row.external_reference,
            amount_minor: row.amount_minor,
            currency: row.currency,
            sender_account: row.sender_account,
            recipient_account: row.recipient_account,
            transaction_type,
            occurred_at: row.occurred_at,
        })
    }
}

pub struct TransactionRepository<'a> {
    db: &'a PgPool,
}

impl<'a> TransactionRepository<'a> {
    pub fn new(db: &'a PgPool) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        transaction: &NewTransaction,
    ) -> Result<Transaction, TransactionRepositoryError> {
        let row = sqlx::query_as::<_, TransactionRow>(
            r#"
            INSERT INTO transactions (
                external_reference,
                amount_minor,
                currency,
                sender_account,
                recipient_account,
                transaction_type,
                occurred_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING
                id,
                external_reference,
                amount_minor,
                currency,
                sender_account,
                recipient_account,
                transaction_type,
                occurred_at
            "#,
        )
        .bind(&transaction.external_reference)
        .bind(transaction.amount_minor)
        .bind(&transaction.currency)
        .bind(&transaction.sender_account)
        .bind(&transaction.recipient_account)
        .bind(transaction.transaction_type.as_str())
        .bind(transaction.occurred_at.clone())
        .fetch_one(self.db)
        .await
        .map_err(TransactionRepositoryError::Database)?;

        Transaction::try_from(row)
    }

    pub async fn find_by_id(
        &self,
        id: i64,
    ) -> Result<Option<Transaction>, TransactionRepositoryError> {
        let row = sqlx::query_as::<_, TransactionRow>(
            r#"
            SELECT
                id,
                external_reference,
                amount_minor,
                currency,
                sender_account,
                recipient_account,
                transaction_type,
                occurred_at
            FROM transactions
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(self.db)
        .await
        .map_err(TransactionRepositoryError::Database)?;

        match row {
            Some(row) => Ok(Some(Transaction::try_from(row)?)),
            None => Ok(None),
        }
    }

    /// Return recent transactions, limiting the result to 1–100 rows.
    pub async fn list_recent(
        &self,
        limit: i64,
    ) -> Result<Vec<Transaction>, TransactionRepositoryError> {
        let limit = limit.clamp(1, 100);

        let rows = sqlx::query_as::<_, TransactionRow>(
            r#"
            SELECT
                id,
                external_reference,
                amount_minor,
                currency,
                sender_account,
                recipient_account,
                transaction_type,
                occurred_at
            FROM transactions
            ORDER BY occurred_at DESC, id DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(self.db)
        .await
        .map_err(TransactionRepositoryError::Database)?;

        rows.into_iter().map(Transaction::try_from).collect()
    }
}
