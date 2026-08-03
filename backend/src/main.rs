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
        email: String::from("alice@finsight.ai"),
        years_experience: 8,
    };

    let case = Case {
        id: 10011,
        title: String::from("International Money Laundering"),
        risk_score: 92,
        status: CaseStatus::Open,
    };

    services::display_case(&case);
    services::display_investigator(&investigator);
    services::display_status(&case.status);
    services::process_case(&case);

    println!("{}", config::APP_NAME);

    println!("{}", investigator.name);

    println!("{}", case.title);
}
