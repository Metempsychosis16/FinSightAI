use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RiskIndicator {
    pub code: String,
    pub description: String,
    pub points: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct RiskFactorExplanation {
    pub code: String,
    pub description: String,
    pub points: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct RiskExplanation {
    pub summary: String,
    pub raw_points: u16,
    pub factors: Vec<RiskFactorExplanation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RiskAssessment {
    pub case_id: i64,
    pub score: u8,
    pub level: RiskLevel,
    pub indicators: Vec<RiskIndicator>,
    pub explanation: RiskExplanation,
}
