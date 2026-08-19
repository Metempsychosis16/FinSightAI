use crate::models::case::Case;
use crate::models::investigator::Investigator;
use crate::models::status::CaseStatus;
use crate::models::validation::Validate;

pub fn display_status(status: &CaseStatus) {
    match status {
        CaseStatus::Open => {
            println!("Case is open.");
        }

        CaseStatus::UnderInvestigation => {
            println!("Investigation in progress.");
        }
        CaseStatus::Esclated => {
            println!("Escalted to senior Investigation");
        }

        CaseStatus::Closed => {
            println!("Case Closed.");
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

        CaseStatus::UnderInvestigation => {
            println!("Collect financial evidence");
        }

        CaseStatus::Esclated => {
            println!("Notify senior investigator");
        }

        CaseStatus::Closed => {
            println!("Archive records.");
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
