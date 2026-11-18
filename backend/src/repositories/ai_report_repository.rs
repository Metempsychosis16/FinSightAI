use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AiReportRecord {
    pub id: i64,
    pub case_id: i64,
    pub generated_by: i64,
    pub provider: String,
    pub model: String,
    pub prompt_version: String,
    pub schema_version: String,
    pub structured_report: Value,
    pub markdown_report: String,
    pub created_at: DateTime<Utc>,
}

pub struct NewAiReport {
    pub case_id: i64,
    pub generated_by: i64,
    pub provider: String,
    pub model: String,
    pub prompt_version: String,
    pub schema_version: String,
    pub structured_report: Value,
    pub markdown_report: String,
}

#[derive(Debug)]
pub enum AiReportRepositoryError {
    Database(sqlx::Error),
    CaseNotFound,
}

pub struct AiReportRepository<'a> {
    db: &'a PgPool,
}

impl<'a> AiReportRepository<'a> {
    pub fn new(db: &'a PgPool) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        report: &NewAiReport,
    ) -> Result<AiReportRecord, AiReportRepositoryError> {
        sqlx::query_as::<_, AiReportRecord>(
            r#"
            INSERT INTO ai_investigation_reports (
                case_id,
                generated_by,
                provider,
                model,
                prompt_version,
                schema_version,
                structured_report,
                markdown_report
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING
                id,
                case_id,
                generated_by,
                provider,
                model,
                prompt_version,
                schema_version,
                structured_report,
                markdown_report,
                created_at
            "#,
        )
        .bind(report.case_id)
        .bind(report.generated_by)
        .bind(&report.provider)
        .bind(&report.model)
        .bind(&report.prompt_version)
        .bind(&report.schema_version)
        .bind(&report.structured_report)
        .bind(&report.markdown_report)
        .fetch_one(self.db)
        .await
        .map_err(AiReportRepositoryError::Database)
    }

    pub async fn list_by_case(
        &self,
        case_id: i64,
    ) -> Result<Vec<AiReportRecord>, AiReportRepositoryError> {
        let case_exists =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM cases WHERE id = $1)")
                .bind(case_id)
                .fetch_one(self.db)
                .await
                .map_err(AiReportRepositoryError::Database)?;

        if !case_exists {
            return Err(AiReportRepositoryError::CaseNotFound);
        }

        sqlx::query_as::<_, AiReportRecord>(
            r#"
            SELECT
                id,
                case_id,
                generated_by,
                provider,
                model,
                prompt_version,
                schema_version,
                structured_report,
                markdown_report,
                created_at
            FROM ai_investigation_reports
            WHERE case_id = $1
            ORDER BY created_at DESC, id DESC
            "#,
        )
        .bind(case_id)
        .fetch_all(self.db)
        .await
        .map_err(AiReportRepositoryError::Database)
    }
}
