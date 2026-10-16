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
fn open_can_transition_to_under_review() {
    assert!(CaseStatus::Open.can_transition_to(CaseStatus::UnderReview));
}

#[test]
fn open_cannot_transition_directly_to_closed() {
    assert!(!CaseStatus::Open.can_transition_to(CaseStatus::Closed));
}

#[test]
fn under_review_can_transition_to_escalated() {
    assert!(CaseStatus::UnderReview.can_transition_to(CaseStatus::Escalated));
}

#[test]
fn escalated_can_transition_to_closed() {
    assert!(CaseStatus::Escalated.can_transition_to(CaseStatus::Closed));
}

#[test]
fn archived_cannot_transition_to_open() {
    assert!(!CaseStatus::Archived.can_transition_to(CaseStatus::Open));
}
