use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaseStatus {
    #[serde(rename = "OPEN")]
    Open,

    #[serde(rename = "UNDER_REVIEW")]
    UnderReview,

    #[serde(rename = "ESCLATED")]
    Esclated,

    #[serde(rename = "CLOSED")]
    Closed,

    #[serde(rename = "ARCHIVED")]
    Archived,
}
