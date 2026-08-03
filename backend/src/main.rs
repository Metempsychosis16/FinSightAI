mod config;
mod models;
mod services;
mod utils;

use models::case::Case;
use models::investigator::Investigator;
use models::status::CaseStatus;

fn main() {
    let investigator = Investigator {
        id: 1,
        name: String::from("Alice"),
        department: String::from("Financial Crimes"),
        email: Some(String::from("alice@finsight.ai")),
        years_experience: 8,
    };

    let case = Case {
        id: 10011,
        title: String::from("International Money Laundering"),
        risk_score: 92,
        status: CaseStatus::Open,
    };

    investigator.display();
    case.display();
    println!("Senior Investigator: {}", investigator.is_senior());
    println!("High Risk Case: {} ", case.is_high_risk());
    services::display_status(&case.status);
    services::process_case(&case);

    println!("{}", config::APP_NAME);

    println!("{}", investigator.name);

    println!("{}", case.title);
}
