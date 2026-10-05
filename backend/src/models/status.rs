use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaseStatus {
    #[serde(rename = "OPEN")]
    Open,

    #[serde(rename = "UNDER_REVIEW")]
    UnderReview,

    #[serde(rename = "ESCALATED")]
    Escalated,

    #[serde(rename = "CLOSED")]
    Closed,

    #[serde(rename = "ARCHIVED")]
    Archived,
}

impl CaseStatus {
    pub fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Open, Self::UnderReview)
                | (Self::UnderReview, Self::Escalated)
                | (Self::UnderReview, Self::Closed)
                | (Self::Escalated, Self::Closed)
                | (Self::Closed, Self::Archived)
        )
    }
    pub fn transition_to(self, next: Self) -> Result<Self, &'static str> {
        if self.can_transition_to(next) {
            Ok(next)
        } else {
            Err("invalid case status transition")
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "OPEN",
            Self::UnderReview => "UNDER_REVIEW",
            Self::Escalated => "ESCALATED",
            Self::Closed => "CLOSED",
            Self::Archived => "ARCHIVED",
        }
    }

    pub fn from_str(value: &str) -> Result<Self, &'static str> {
        match value {
            "OPEN" => Ok(Self::Open),
            "UNDER_REVIEW" => Ok(Self::UnderReview),
            "ESCALATED" => Ok(Self::Escalated),
            "CLOSED" => Ok(Self::Closed),
            "ARCHIVED" => Ok(Self::Archived),
            _ => Err("invalid case status"),
        }
    }
}

#[test]
fn open_converts_to_database_value() {
    assert_eq!(CaseStatus::Open.as_str(), "OPEN");
}

#[test]
fn under_review_converts_to_database_value() {
    assert_eq!(CaseStatus::UnderReview.as_str(), "UNDER_REVIEW");
}

#[test]
fn database_value_converts_to_open() {
    assert_eq!(CaseStatus::from_str("OPEN"), Ok(CaseStatus::Open));
}

#[test]
fn database_value_converts_to_under_review() {
    assert_eq!(
        CaseStatus::from_str("UNDER_REVIEW"),
        Ok(CaseStatus::UnderReview)
    );
}

#[test]
fn invalid_database_status_is_rejected() {
    assert!(CaseStatus::from_str("BANANA").is_err());
}
