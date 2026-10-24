use chrono::{DateTime, Utc};

use super::validation::Validate;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Investigator {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub department: String,
    pub years_experience: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Investigator {
    pub fn is_senior(&self) -> bool {
        self.years_experience >= 5
    }
}

impl Validate for Investigator {
    fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err(String::from("Investigator name cannot be empty"));
        }

        if self.department.trim().is_empty() {
            return Err(String::from("Investigator department cannot be empty"));
        }

        if !(0..60).contains(&self.years_experience) {
            return Err(String::from("Years of experience must be between 0 and 59"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Investigator;
    use crate::models::validation::Validate;

    fn sample_investigator() -> Investigator {
        let now = chrono::Utc::now();

        Investigator {
            id: 1,
            user_id: 10,
            name: String::from("Alex Morgan"),
            department: String::from("Financial Crime"),
            years_experience: 6,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn accepts_a_valid_investigator() {
        assert!(sample_investigator().validate().is_ok());
    }

    #[test]
    fn rejects_an_empty_name() {
        let mut investigator = sample_investigator();
        investigator.name = String::from("   ");

        assert!(investigator.validate().is_err());
    }

    #[test]
    fn rejects_an_empty_department() {
        let mut investigator = sample_investigator();
        investigator.department = String::new();

        assert!(investigator.validate().is_err());
    }

    #[test]
    fn rejects_unrealistic_experience() {
        let mut investigator = sample_investigator();
        investigator.years_experience = 60;

        assert!(investigator.validate().is_err());
    }

    #[test]
    fn identifies_senior_investigators() {
        let investigator = sample_investigator();
        assert!(investigator.is_senior());
    }
}
