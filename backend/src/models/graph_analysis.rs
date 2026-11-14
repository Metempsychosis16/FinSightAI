use std::collections::BTreeSet;

use serde::Serialize;

use crate::models::transaction_graph::TransactionGraph;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AccountAnalysis {
    pub account_id: String,
    pub incoming_count: usize,
    pub outgoing_count: usize,
    pub unique_senders: usize,
    pub unique_recipients: usize,
    pub total_connections: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphAnalysis {
    pub case_id: i64,
    pub account_count: usize,
    pub transaction_count: usize,
    pub accounts: Vec<AccountAnalysis>,
}

impl GraphAnalysis {
    pub fn from_graph(graph: &TransactionGraph) -> Self {
        let mut accounts = Vec::new();

        for node in &graph.nodes {
            let account_id = &node.account_id;

            let mut incoming_count = 0;
            let mut outgoing_count = 0;

            let mut senders = BTreeSet::new();
            let mut recipients = BTreeSet::new();

            for edge in &graph.edges {
                if edge.to_account == *account_id {
                    incoming_count += 1;
                    senders.insert(edge.from_account.clone());
                }

                if edge.from_account == *account_id {
                    outgoing_count += 1;
                    recipients.insert(edge.to_account.clone());
                }
            }

            let total_connections = senders.union(&recipients).count();

            accounts.push(AccountAnalysis {
                account_id: account_id.clone(),
                incoming_count,
                outgoing_count,
                unique_senders: senders.len(),
                unique_recipients: recipients.len(),
                total_connections,
            });
        }

        Self {
            case_id: graph.case_id,
            account_count: graph.nodes.len(),
            transaction_count: graph.edges.len(),
            accounts,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    use crate::models::transaction_graph::{AccountNode, TransactionEdge};

    fn sample_graph() -> TransactionGraph {
        TransactionGraph {
            case_id: 42,
            nodes: vec![
                AccountNode {
                    account_id: "A".to_string(),
                },
                AccountNode {
                    account_id: "B".to_string(),
                },
                AccountNode {
                    account_id: "C".to_string(),
                },
            ],
            edges: vec![
                TransactionEdge {
                    transaction_id: 1,
                    external_reference: "TX001".to_string(),
                    from_account: "A".to_string(),
                    to_account: "B".to_string(),
                    amount_minor: 10000,
                    currency: "INR".to_string(),
                    transaction_type: "transfer".to_string(),
                    occurred_at: Utc::now(),
                },
                TransactionEdge {
                    transaction_id: 2,
                    external_reference: "TX002".to_string(),
                    from_account: "A".to_string(),
                    to_account: "C".to_string(),
                    amount_minor: 20000,
                    currency: "INR".to_string(),
                    transaction_type: "transfer".to_string(),
                    occurred_at: Utc::now(),
                },
                TransactionEdge {
                    transaction_id: 3,
                    external_reference: "TX003".to_string(),
                    from_account: "B".to_string(),
                    to_account: "C".to_string(),
                    amount_minor: 30000,
                    currency: "INR".to_string(),
                    transaction_type: "transfer".to_string(),
                    occurred_at: Utc::now(),
                },
            ],
        }
    }

    #[test]
    fn calculates_incoming_and_outgoing_counts() {
        let graph = sample_graph();
        let analysis = GraphAnalysis::from_graph(&graph);

        let account_a = analysis
            .accounts
            .iter()
            .find(|account| account.account_id == "A")
            .unwrap();

        assert_eq!(account_a.outgoing_count, 2);
        assert_eq!(account_a.incoming_count, 0);
    }

    #[test]
    fn calculates_unique_senders_and_recipients() {
        let graph = sample_graph();
        let analysis = GraphAnalysis::from_graph(&graph);

        let account_c = analysis
            .accounts
            .iter()
            .find(|account| account.account_id == "C")
            .unwrap();

        assert_eq!(account_c.unique_senders, 2);
        assert_eq!(account_c.incoming_count, 2);
        assert_eq!(account_c.outgoing_count, 0);
    }

    #[test]
    fn calculates_total_connections_without_duplicates() {
        let graph = sample_graph();
        let analysis = GraphAnalysis::from_graph(&graph);

        let account_b = analysis
            .accounts
            .iter()
            .find(|account| account.account_id == "B")
            .unwrap();

        assert_eq!(account_b.total_connections, 2);
    }

    #[test]
    fn includes_graph_summary_counts() {
        let graph = sample_graph();
        let analysis = GraphAnalysis::from_graph(&graph);

        assert_eq!(analysis.case_id, 42);
        assert_eq!(analysis.account_count, 3);
        assert_eq!(analysis.transaction_count, 3);
    }
}
