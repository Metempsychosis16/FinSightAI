use super::validation::Validate;

pub struct Investigator {
    pub id: u32,
    pub name: String,
    pub department: String,
    pub email: Option<String>,

    pub years_experience: u8,
}

impl Investigator {
    pub fn display(&self) {
        println!("Investigator : {} ", self.name);

        match &self.email {
            Some(email) => println!("Email: {} ", email),
            None => println!("Email:Not Assigned"),
        }
        println!("Department : {} ", self.department);
        println!("Experience : {} ", self.years_experience);
    }

    pub fn is_senior(&self) -> bool {
        self.years_experience >= 5
    }
}

impl Validate for Investigator {
    fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err(String::from("Investigator name cannot be empty"));
        }

        if self.years_experience >= 60 {
            return Err(String::from("Years of experience is unrelistic"));
        }
        Ok(())
    }
}
