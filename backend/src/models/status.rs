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

impl CaseStatus {
    pub fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Open, Self::UnderReview)
                | (Self::UnderReview, Self::Esclated)
                | (Self::UnderReview, Self::Closed)
                | (Self::Esclated, Self::Closed)
                | (Self::Closed, Self::Archived)
        )
    }
    pub fn transitin_to(self, next: Self) -> Result<Self, &'static str> {
        if self.can_transition_to(next) {
            Ok(next)
        } else {
            Err("invalid case status transition")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CaseStatus;

    #[test]
    fn open_can_move_to_under_review() {
        assert!(CaseStatus::Open.can_transition_to(CaseStatus::UnderReview));
    }

    #[test]
    fn open_cannot_move_directly_to_closed() {
        assert!(!CaseStatus::Open.can_transition_to(CaseStatus::Closed));
    }

    #[test]
    fn under_review_can_be_escalated() {
        assert!(CaseStatus::UnderReview.can_transition_to(CaseStatus::Esclated));
    }

    #[test]
    fn esclated_can_be_closed() {
        assert!(CaseStatus::Esclated.can_transition_to(CaseStatus::Closed));
    }

    #[test]
    fn closed_can_be_archived() {
        assert!(CaseStatus::Closed.can_transition_to(CaseStatus::Archived));
    }

    #[test]
    fn archived_cannot_move_anywhere() {
        assert!(!CaseStatus::Archived.can_transition_to(CaseStatus::Open));
    }
}
