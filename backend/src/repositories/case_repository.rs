use crate::models::case_response::CaseRow;
use sqlx::PgPool;

pub struct CaseRepository<'a> {
    db: &'a PgPool,
}

impl<'a> CaseRepository<'a> {
    pub fn new(db: &'a PgPool) -> Self {
        Self { db }
    }

    pub async fn find_status(&self, case_id: i64) -> Result<Option<String>, sqlx::Error> {
        sqlx::query_scalar::<_, String>(
            r#"
            SELECT status
            FROM cases
            WHERE id = $1
            "#,
        )
        .bind(case_id)
        .fetch_optional(self.db)
        .await
    }

    pub async fn update_status(&self, case_id: i64, status: &str) -> Result<CaseRow, sqlx::Error> {
        sqlx::query_as::<_, CaseRow>(
            r#"
            UPDATE cases
            SET status = $1,
                updated_at = NOW()
            WHERE id = $2
            RETURNING id, title, risk_score, status, created_at, updated_at
            "#,
        )
        .bind(status)
        .bind(case_id)
        .fetch_one(self.db)
        .await
    }
}
