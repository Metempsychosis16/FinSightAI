use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::models::case_response::CaseResponse;
use crate::models::evidence::Evidence;

#[derive(Debug, Serialize)]
pub struct DatasetInvestigator {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub department: String,
    pub years_experience: i16,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct DatasetTransaction {
    pub id: i64,
    pub external_reference: String,
    pub amount_minor: i64,
    pub currency: String,
    pub sender_account: String,
    pub recipient_account: String,
    pub transaction_type: String,
    pub occurred_at: DateTime<Utc>,
    pub linked_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct InvestigationDataset {
    pub case: CaseResponse,
    pub investigator: Option<DatasetInvestigator>,
    pub transaction_count: usize,
    pub transactions: Vec<DatasetTransaction>,
    pub evidence_count: usize,
    pub evidence: Vec<Evidence>,
}
