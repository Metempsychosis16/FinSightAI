use crate::models::case_response::CaseRow;
use crate::models::status::CaseStatus;
use sqlx::PgPool;

#[derive(Debug)]
pub enum CaseServiceError {
    Database(sqlx::Error),
    NotFound,
    InvalidStoredStatus,
    InvalidTransition,
}

pub struct CaseService<'a> {
    db: &'a PgPool,
}

impl<'a> CaseService<'a> {
    pub fn new(db: &'a PgPool) -> Self {
        Self { db }
    }

    pub async fn update_status(
        &self,
        case_id: i64,
        next_status: CaseStatus,
    ) -> Result<CaseRow, CaseServiceError> {
        let current_status = sqlx::query_scalar::<_, String>(
            r#"
        SELECT status
        FROM cases
        WHERE id = $1
        "#,
        )
        .bind(case_id)
        .fetch_optional(self.db)
        .await
        .map_err(CaseServiceError::Database)?;

        let current_status = match current_status {
            Some(status) => {
                CaseStatus::from_str(&status).map_err(|_| CaseServiceError::InvalidStoredStatus)?
            }
            None => return Err(CaseServiceError::NotFound),
        };

        current_status
            .transition_to(next_status)
            .map_err(|_| CaseServiceError::InvalidTransition)?;

        let row = sqlx::query_as::<_, CaseRow>(
            r#"
        UPDATE cases
        SET status = $1,
            updated_at = NOW()
        WHERE id = $2
        RETURNING id, title, risk_score, status, created_at, updated_at
        "#,
        )
        .bind(next_status.as_str())
        .bind(case_id)
        .fetch_one(self.db)
        .await
        .map_err(CaseServiceError::Database)?;

        Ok(row)
    }
}
