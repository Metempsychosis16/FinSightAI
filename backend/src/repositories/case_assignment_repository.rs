use sqlx::PgPool;

pub struct CaseAssignmentRepository<'a> {
    db: &'a PgPool,
}

impl<'a> CaseAssignmentRepository<'a> {
    pub fn new(db: &'a PgPool) -> Self {
        Self { db }
    }

    pub async fn investigator_is_assignable(
        &self,
        investigator_id: i64,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM investigators AS i
                JOIN users AS u ON u.id = i.user_id
                WHERE i.id = $1
                  AND u.is_active = TRUE
                  AND u.role IN ('INVESTIGATOR', 'SENIOR_INVESTIGATOR')
            )
            "#,
        )
        .bind(investigator_id)
        .fetch_one(self.db)
        .await
    }

    pub async fn assign(&self, case_id: i64, investigator_id: i64) -> Result<bool, sqlx::Error> {
        let updated_case = sqlx::query_scalar::<_, i64>(
            r#"
            UPDATE cases
            SET investigator_id = $1,
                updated_at = NOW()
            WHERE id = $2
            RETURNING id
            "#,
        )
        .bind(investigator_id)
        .bind(case_id)
        .fetch_optional(self.db)
        .await?;

        Ok(updated_case.is_some())
    }
}
