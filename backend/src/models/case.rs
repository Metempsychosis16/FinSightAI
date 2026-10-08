use crate::models::status::CaseStatus;
use crate::models::validation::Validate;

pub struct Case {
    pub id: u32,
    pub title: String,
    pub risk_score: u8,
    pub status: CaseStatus,
}

impl Case {
    pub fn display(&self) {
        println!("Case : {} ", self.title);
        println!("Risk Score : {} ", self.risk_score);
    }

    pub fn is_high_risk(&self) -> bool {
        self.risk_score >= 80
    }
}

impl Validate for Case {
    fn validate(&self) -> Result<(), String> {
        if self.id <= 0 {
            return Err(String::from("case id must be greater than 0"));
        }
        if self.title.trim().is_empty() {
            return Err(String::from("case title cannot be empty"));
        }

        if !(0..=100).contains(&self.risk_score) {
            return Err(String::from("risk score must be between 0 adn 100"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::status::CaseStatus;
    use crate::models::validation::Validate;

    fn valid_case() -> Case {
        Case {
            id: 1,
            title: String::from("Suspicious transaction investigation"),
            risk_score: 75,
            status: CaseStatus::Open,
        }
    }

    #[test]
    fn valid_case_passes_validation() {
        let case = valid_case();

        assert!(case.validate().is_ok());
    }

    #[test]
    fn zero_id_is_rejected() {
        let mut case = valid_case();
        case.id = 0;

        assert_eq!(
            case.validate(),
            Err(String::from("case id must be greater than 0"))
        );
    }

    #[test]
    fn empty_title_is_rejected() {
        let mut case = valid_case();
        case.title = String::new();

        assert_eq!(
            case.validate(),
            Err(String::from("case title cannot be empty"))
        );
    }

    #[test]
    fn whitespace_title_is_rejected() {
        let mut case = valid_case();
        case.title = String::from("   ");

        assert_eq!(
            case.validate(),
            Err(String::from("case title cannot be empty"))
        );
    }

    #[test]
    fn negative_risk_score_is_rejected() {
        let mut case = valid_case();
        case.risk_score = -1;

        assert_eq!(
            case.validate(),
            Err(String::from("risk score must be between 0 and 100"))
        );
    }

    #[test]
    fn risk_score_above_100_is_rejected() {
        let mut case = valid_case();
        case.risk_score = 101;

        assert_eq!(
            case.validate(),
            Err(String::from("risk score must be between 0 and 100"))
        );
    }

    #[test]
    fn risk_score_boundaries_are_valid() {
        let mut case = valid_case();

        case.risk_score = 0;
        assert!(case.validate().is_ok());

        case.risk_score = 100;
        assert!(case.validate().is_ok());
    }
}
