use crate::{
    models::{
        graph_analysis::GraphAnalysis, investigation_dataset::InvestigationDataset,
        transaction_graph::TransactionGraph,
    },
    risk::engine::{FinancialCrimeRiskEngine, RiskEngine},
};

use super::evidence_package::{EvidenceFact, InvestigationEvidencePackage, TransactionFact};

/// Collects existing investigation facts into an AI-ready package.
///
/// This component does not communicate with an LLM and does not
/// modify database records or risk scores.
pub struct EvidenceCollector;

impl EvidenceCollector {
    pub fn collect(dataset: &InvestigationDataset) -> InvestigationEvidencePackage {
        // Convert dataset transactions into deliberately selected facts.
        let transactions: Vec<TransactionFact> = dataset
            .transactions
            .iter()
            .map(TransactionFact::from)
            .collect();

        // Convert evidence database records into selected metadata.
        let evidence: Vec<EvidenceFact> = dataset.evidence.iter().map(EvidenceFact::from).collect();

        // Build the transaction graph using the existing implementation.
        let graph = TransactionGraph::from_dataset(dataset);

        // Calculate the account-level graph analysis.
        let graph_analysis = GraphAnalysis::from_graph(&graph);

        // Reuse the deterministic financial crime risk engine.
        let risk_assessment = FinancialCrimeRiskEngine::new().assess(dataset);

        InvestigationEvidencePackage::new(
            dataset.case.id,
            dataset.transaction_count,
            transactions,
            dataset.evidence_count,
            evidence,
            graph_analysis,
            risk_assessment,
        )
    }
}
