use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use super::evidence_package::InvestigationEvidencePackage;

pub const OUTPUT_SCHEMA_VERSION: &str = "1.1";

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct AnalysisFinding {
    pub description: String,

    pub rationale: String,

    #[serde(default)]
    pub limitations: Vec<String>,

    #[serde(default)]
    pub transaction_ids: Vec<i64>,

    #[serde(default)]
    pub evidence_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredInvestigationReport {
    pub schema_version: String,
    pub case_id: i64,
    pub executive_summary: String,

    #[serde(default)]
    pub verified_facts: Vec<AnalysisFinding>,

    #[serde(default)]
    pub risk_engine_findings: Vec<AnalysisFinding>,

    #[serde(default)]
    pub analytical_inferences: Vec<AnalysisFinding>,

    #[serde(default)]
    pub missing_information: Vec<String>,

    #[serde(default)]
    pub follow_up_questions: Vec<String>,

    #[serde(default)]
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputValidationError {
    UnsupportedSchemaVersion,
    CaseIdMismatch,
    UnknownTransactionId(i64),
    UnknownEvidenceId(i64),
}

impl std::fmt::Display for OutputValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion => {
                write!(f, "Unsupported AI output schema version")
            }
            Self::CaseIdMismatch => {
                write!(f, "AI report case ID does not match the requested case")
            }
            Self::UnknownTransactionId(id) => {
                write!(f, "AI report references unknown transaction ID {id}")
            }
            Self::UnknownEvidenceId(id) => {
                write!(f, "AI report references unknown evidence ID {id}")
            }
        }
    }
}

impl std::error::Error for OutputValidationError {}

impl StructuredInvestigationReport {
    pub fn validate_against(
        &self,
        package: &InvestigationEvidencePackage,
    ) -> Result<(), OutputValidationError> {
        if self.schema_version != OUTPUT_SCHEMA_VERSION {
            return Err(OutputValidationError::UnsupportedSchemaVersion);
        }

        if self.case_id != package.case_id {
            return Err(OutputValidationError::CaseIdMismatch);
        }

        let transaction_ids: HashSet<i64> =
            package.transactions.iter().map(|item| item.id).collect();

        let evidence_ids: HashSet<i64> = package.evidence.iter().map(|item| item.id).collect();

        let findings = self
            .verified_facts
            .iter()
            .chain(self.risk_engine_findings.iter())
            .chain(self.analytical_inferences.iter());

        for finding in findings {
            for id in &finding.transaction_ids {
                if !transaction_ids.contains(id) {
                    return Err(OutputValidationError::UnknownTransactionId(*id));
                }
            }

            for id in &finding.evidence_ids {
                if !evidence_ids.contains(id) {
                    return Err(OutputValidationError::UnknownEvidenceId(*id));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{AnalysisFinding, OUTPUT_SCHEMA_VERSION, StructuredInvestigationReport};

    use crate::{
        ai::evidence_package::{EvidenceFact, InvestigationEvidencePackage, TransactionFact},
        models::graph_analysis::GraphAnalysis,
        risk::models::{RiskAssessment, RiskExplanation, RiskLevel},
    };
    use chrono::Utc;

    fn sample_package() -> InvestigationEvidencePackage {
        let now = Utc::now();

        let transactions = vec![TransactionFact {
            id: 10,
            external_reference: "TX-10".to_owned(),
            amount_minor: 125000,
            currency: "INR".to_owned(),
            sender_account: "ACCOUNT-A".to_owned(),
            recipient_account: "ACCOUNT-B".to_owned(),
            transaction_type: "TRANSFER".to_owned(),
            occurred_at: now,
        }];

        let evidence = vec![EvidenceFact {
            id: 20,
            title: "Bank statement".to_owned(),
            description: "Statement metadata".to_owned(),
            evidence_type: "DOCUMENT".to_owned(),
            has_sha256_hash: true,
            has_source_reference: true,
        }];

        let graph_analysis = GraphAnalysis {
            case_id: 42,
            account_count: 2,
            transaction_count: 1,
            accounts: vec![],
        };

        let risk_assessment = RiskAssessment {
            case_id: 42,
            score: 25,
            level: RiskLevel::Medium,
            indicators: vec![],
            explanation: RiskExplanation {
                summary: "Sample risk assessment".to_owned(),
                raw_points: 25,
                factors: vec![],
            },
        };

        InvestigationEvidencePackage::new(
            42,
            1,
            transactions,
            1,
            evidence,
            graph_analysis,
            risk_assessment,
        )
    }

    fn sample_report() -> StructuredInvestigationReport {
        StructuredInvestigationReport {
            schema_version: OUTPUT_SCHEMA_VERSION.to_owned(),
            case_id: 42,
            executive_summary: "Sample investigation report".to_owned(),
            verified_facts: vec![],
            risk_engine_findings: vec![],
            analytical_inferences: vec![],
            missing_information: vec![],
            follow_up_questions: vec![],
            limitations: vec![],
        }
    }

    #[test]
    fn accepts_references_that_exist_in_the_package() {
        let package = sample_package();
        let mut report = sample_report();

        report.verified_facts.push(AnalysisFinding {
            description: "Transaction 10 is included in the dataset".to_owned(),
            rationale: "Transaction 10 appears in the supplied evidence package.".to_owned(),
            limitations: vec![
                "Its inclusion does not establish whether the transaction was suspicious."
                    .to_owned(),
            ],

            transaction_ids: vec![10],
            evidence_ids: vec![20],
        });

        assert!(report.validate_against(&package).is_ok());
    }

    #[test]
    fn rejects_fabricated_transaction_id() {
        let package = sample_package();
        let mut report = sample_report();

        report.analytical_inferences.push(AnalysisFinding {
            description: "A suspicious transaction pattern".to_owned(),

            rationale: "Transaction 10 appears in the supplied evidence package.".to_owned(),
            limitations: vec![
                "Its inclusion does not establish whether the transaction was suspicious."
                    .to_owned(),
            ],

            transaction_ids: vec![9999],
            evidence_ids: vec![],
        });

        assert_eq!(
            report.validate_against(&package),
            Err(super::OutputValidationError::UnknownTransactionId(9999))
        );
    }

    #[test]
    fn rejects_fabricated_evidence_id() {
        let package = sample_package();
        let mut report = sample_report();

        report.verified_facts.push(AnalysisFinding {
            description: "A statement supports this finding".to_owned(),

            rationale: "Transaction 10 appears in the supplied evidence package.".to_owned(),
            limitations: vec![
                "Its inclusion does not establish whether the transaction was suspicious."
                    .to_owned(),
            ],
            transaction_ids: vec![],
            evidence_ids: vec![8888],
        });

        assert_eq!(
            report.validate_against(&package),
            Err(super::OutputValidationError::UnknownEvidenceId(8888))
        );
    }

    #[test]
    fn rejects_report_for_a_different_case() {
        let package = sample_package();
        let mut report = sample_report();

        report.case_id = 99;

        assert_eq!(
            report.validate_against(&package),
            Err(super::OutputValidationError::CaseIdMismatch)
        );
    }

    #[test]
    fn structured_report_deserializes_valid_json() {
        let json = r#"
        {
            "schema_version": "1.1",
            "case_id": 42,
            "executive_summary": "Three transactions were supplied.",
            "verified_facts": [
                {
                    "description": "Transaction 10 was included.",

                    "rationale": "Transaction 10 appears in the supplied investigation data.",
    "limitations": [
        "Its inclusion does not establish whether the transaction was suspicious."
    ],

                    "transaction_ids": [10],
                    "evidence_ids": []
                }
            ],
            "risk_engine_findings": [],
            "analytical_inferences": [],
            "missing_information": [],
            "follow_up_questions": [],
            "limitations": []
        }
        "#;

        let report: StructuredInvestigationReport =
            serde_json::from_str(json).expect("valid report should deserialize");

        assert_eq!(report.schema_version, OUTPUT_SCHEMA_VERSION);
        assert_eq!(report.case_id, 42);
        assert_eq!(report.verified_facts.len(), 1);
        assert_eq!(report.verified_facts[0].transaction_ids, vec![10]);
    }

    #[test]
    fn structured_report_rejects_wrong_field_types() {
        let json = r#"
        {
            "schema_version": "1.1",
            "case_id": "not-an-integer",
            "executive_summary": "Summary",
            "verified_facts": [],
            "risk_engine_findings": [],
            "analytical_inferences": [],
            "missing_information": [],
            "follow_up_questions": [],
            "limitations": []
        }
        "#;

        assert!(serde_json::from_str::<StructuredInvestigationReport>(json).is_err());
    }

    #[test]
    fn finding_can_reference_multiple_records() {
        let finding = AnalysisFinding {
            description: "A transaction pattern".to_owned(),

            rationale: "Transaction 10 appears in the supplied evidence package.".to_owned(),
            limitations: vec![
                "Its inclusion does not establish whether the transaction was suspicious."
                    .to_owned(),
            ],

            transaction_ids: vec![10, 11],
            evidence_ids: vec![20],
        };

        assert_eq!(finding.transaction_ids.len(), 2);
        assert_eq!(finding.evidence_ids, vec![20]);
    }
}
