use crate::models::status::CaseStatus;
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, sqlx::FromRow)]
pub struct CaseRow {
    pub id: i64,
    pub title: String,
    pub risk_score: i16,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CaseResponse {
    pub id: i64,
    pub title: String,
    pub risk_score: i16,
    pub status: CaseStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub enum CaseResponseError {
    InvalidStatus,
}

impl TryFrom<CaseRow> for CaseResponse {
    type Error = CaseResponseError;

    fn try_from(row: CaseRow) -> Result<Self, Self::Error> {
        let status =
            CaseStatus::from_str(&row.status).map_err(|_| CaseResponseError::InvalidStatus)?;

        Ok(Self {
            id: row.id,
            title: row.title,
            risk_score: row.risk_score,
            status,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
