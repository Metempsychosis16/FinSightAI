use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::{
    models::{
        evidence::Evidence, graph_analysis::GraphAnalysis,
        investigation_dataset::DatasetTransaction,
    },
    risk::models::RiskAssessment,
};

/// A transaction fact selected for AI-assisted investigation.
#[derive(Debug, Clone, Serialize)]
pub struct TransactionFact {
    pub id: i64,
    pub external_reference: String,
    pub amount_minor: i64,
    pub currency: String,
    pub sender_account: String,
    pub recipient_account: String,
    pub transaction_type: String,
    pub occurred_at: DateTime<Utc>,
}

impl From<&DatasetTransaction> for TransactionFact {
    fn from(transaction: &DatasetTransaction) -> Self {
        Self {
            id: transaction.id,
            external_reference: transaction.external_reference.clone(),
            amount_minor: transaction.amount_minor,
            currency: transaction.currency.clone(),
            sender_account: transaction.sender_account.clone(),
            recipient_account: transaction.recipient_account.clone(),
            transaction_type: transaction.transaction_type.clone(),
            occurred_at: transaction.occurred_at.clone(),
        }
    }
}

/// Evidence metadata selected for AI-assisted investigation.
///
/// The document itself is NOT included. The description is metadata
/// and must be treated as untrusted input by future prompt code.
#[derive(Debug, Clone, Serialize)]
pub struct EvidenceFact {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub evidence_type: String,
    pub has_sha256_hash: bool,
    pub has_source_reference: bool,
}

impl From<&Evidence> for EvidenceFact {
    fn from(evidence: &Evidence) -> Self {
        Self {
            id: evidence.id,
            title: evidence.title.clone(),
            description: evidence.description.clone(),
            evidence_type: evidence.evidence_type.clone(),
            has_sha256_hash: evidence.sha256_hash.is_some(),
            has_source_reference: !evidence.source_reference.trim().is_empty(),
        }
    }
}

/// All facts collected for one investigation.
///
/// Counts from the dataset are kept separately from the number of
/// records actually included in this package. This makes discrepancies
/// visible instead of silently hiding them.
#[derive(Debug, Serialize)]
pub struct InvestigationEvidencePackage {
    pub case_id: i64,

    pub dataset_transaction_count: usize,
    pub included_transaction_count: usize,
    pub transactions: Vec<TransactionFact>,

    pub dataset_evidence_count: usize,
    pub included_evidence_count: usize,
    pub evidence: Vec<EvidenceFact>,

    pub graph_analysis: GraphAnalysis,
    pub risk_assessment: RiskAssessment,

    pub limitations: Vec<String>,
}

impl InvestigationEvidencePackage {
    pub fn new(
        case_id: i64,
        dataset_transaction_count: usize,
        transactions: Vec<TransactionFact>,
        dataset_evidence_count: usize,
        evidence: Vec<EvidenceFact>,
        graph_analysis: GraphAnalysis,
        risk_assessment: RiskAssessment,
    ) -> Self {
        let included_transaction_count = transactions.len();
        let included_evidence_count = evidence.len();

        Self {
            case_id,
            dataset_transaction_count,
            included_transaction_count,
            transactions,
            dataset_evidence_count,
            included_evidence_count,
            evidence,
            graph_analysis,
            risk_assessment,
            limitations: vec![
                "Evidence document contents have not been fetched or parsed; only metadata is included."
                    .to_owned(),
                "Investigator notes are not included because a notes model and repository have not yet been implemented."
                    .to_owned(),
                "Transaction amounts remain in their original currencies; no currency conversion is performed."
                    .to_owned(),
                "The graph information contains only the analysis implemented in the current graph module; it does not imply unimplemented path or connected-component analysis."
                    .to_owned(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::{EvidenceFact, TransactionFact};
    use crate::models::{evidence::Evidence, investigation_dataset::DatasetTransaction};

    #[test]
    fn transaction_fact_preserves_original_transaction_values() {
        let now = Utc::now();

        let transaction = DatasetTransaction {
            id: 10,
            external_reference: "BANK-TXN-100".to_owned(),
            amount_minor: 125000,
            currency: "INR".to_owned(),
            sender_account: "ACCOUNT-A".to_owned(),
            recipient_account: "ACCOUNT-B".to_owned(),
            transaction_type: "TRANSFER".to_owned(),
            occurred_at: now,
            linked_at: now,
        };

        let fact = TransactionFact::from(&transaction);

        assert_eq!(fact.id, 10);
        assert_eq!(fact.amount_minor, 125000);
        assert_eq!(fact.currency, "INR");
        assert_eq!(fact.sender_account, "ACCOUNT-A");
        assert_eq!(fact.recipient_account, "ACCOUNT-B");
        assert_eq!(fact.external_reference, "BANK-TXN-100");
        assert_eq!(fact.occurred_at, now);
    }

    #[test]
    fn evidence_fact_does_not_expose_internal_storage_details() {
        let now = Utc::now();

        let evidence = Evidence {
            id: 20,
            case_id: 7,
            title: "Bank statement".to_owned(),
            description: "Statement metadata for investigation".to_owned(),
            evidence_type: "DOCUMENT".to_owned(),
            source_reference: "private://internal/storage/statement.pdf".to_owned(),
            sha256_hash: Some("a".repeat(64)),
            collected_by: 99,
            created_at: now,
        };

        let fact = EvidenceFact::from(&evidence);
        let serialized = serde_json::to_string(&fact).expect("evidence fact should serialize");

        assert_eq!(fact.id, 20);
        assert_eq!(fact.title, "Bank statement");
        assert!(fact.has_sha256_hash);
        assert!(fact.has_source_reference);

        assert!(!serialized.contains("private://"));
        assert!(!serialized.contains("collected_by"));
        assert!(!serialized.contains(&"a".repeat(64)));
    }

    #[test]
    fn evidence_without_hash_or_reference_is_marked_correctly() {
        let now = Utc::now();

        let evidence = Evidence {
            id: 21,
            case_id: 7,
            title: "Investigator note attachment".to_owned(),
            description: "Metadata only".to_owned(),
            evidence_type: "DOCUMENT".to_owned(),
            source_reference: "  ".to_owned(),
            sha256_hash: None,
            collected_by: 99,
            created_at: now,
        };

        let fact = EvidenceFact::from(&evidence);

        assert!(!fact.has_sha256_hash);
        assert!(!fact.has_source_reference);
    }
}
