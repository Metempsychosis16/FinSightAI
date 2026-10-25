use crate::models::case::Case;
use crate::models::investigator::Investigator;
use crate::models::status::CaseStatus;
use crate::models::validation::Validate;

pub mod bulk_transaction_service;
pub mod case_assignment_service;
pub mod case_service;
pub mod login_service;
pub mod password_service;
pub mod registration_service;
pub mod transaction_service;

pub fn display_status(status: &CaseStatus) {
    match status {
        CaseStatus::Open => {
            println!("Case is open.");
        }

        CaseStatus::UnderReview => {
            println!("Investigation in progress.");
        }
        CaseStatus::Escalated => {
            println!("Escaalted to senior Investigation");
        }

        CaseStatus::Closed => {
            println!("Case Closed.");
        }

        CaseStatus::Archived => {
            println!("handle archived case");
        }
    }
}

pub fn process_case(case: &Case) {
    match case.status {
        CaseStatus::Open if case.risk_score >= 90 => {
            println!("Immediate investigation required");
        }

        CaseStatus::Open => {
            println!("Assign Investigator")
        }

        CaseStatus::UnderReview => {
            println!("Collect financial evidence");
        }

        CaseStatus::Escalated => {
            println!("Notify senior investigator");
        }

        CaseStatus::Closed => {
            println!("Case Closed.");
        }
        CaseStatus::Archived => {
            println!("Archived Records")
        }
    }
}

pub fn validate_model<T: Validate>(model: &T) {
    match model.validate() {
        Ok(()) => {
            println!("validation successful.");
        }
        Err(error) => {
            println!("Validation failed: {}", error);
        }
    }
}
