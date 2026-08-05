use super::status::CaseStatus;
use super::validation::Validate;

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
        if self.risk_score > 100 {
            Err(String::from("Risk score cannot exceed 100"))
        } else {
            Ok(())
        }
    }
}
