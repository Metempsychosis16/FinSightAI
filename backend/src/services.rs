use crate::models::case::Case;
use crate::models::investigator::Investigator;
use crate::models::status::CaseStatus;

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

pub fn display_investigator(investigator: &Investigator) {
    println!("Name : {} ", investigator.name);

    println!("Department : {} ", investigator.department);
}

pub fn display_case(case: &Case) {
    println!("Case : {} ", case.title);
    println!("Risk : {} ", case.risk_score);
}
