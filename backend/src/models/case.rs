use super::status::CaseStatus;

pub struct Case {
    pub id: u32,
    pub title: String,
    pub risk_score: u8,
    pub status: CaseStatus,
}
