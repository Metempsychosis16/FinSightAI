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
