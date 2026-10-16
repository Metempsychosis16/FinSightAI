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
