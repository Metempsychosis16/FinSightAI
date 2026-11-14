use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::models::investigation_dataset::InvestigationDataset;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AccountNode {
    pub account_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransactionEdge {
    pub transaction_id: i64,
    pub external_reference: String,
    pub from_account: String,
    pub to_account: String,
    pub amount_minor: i64,
    pub currency: String,
    pub transaction_type: String,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AccountRelationship {
    pub from_account: String,
    pub to_account: String,
    pub transaction_count: usize,
    pub transaction_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransactionGraph {
    pub case_id: i64,
    pub nodes: Vec<AccountNode>,
    pub edges: Vec<TransactionEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TransactionPath {
    pub accounts: Vec<String>,
    pub hops: usize,
}

#[derive(Debug, Serialize)]
pub struct TransactionGraphResponse {
    pub case_id: i64,
    pub nodes: Vec<AccountNode>,
    pub edges: Vec<TransactionEdge>,
    pub relationships: Vec<AccountRelationship>,
    pub adjacency_list: BTreeMap<String, Vec<String>>,
}

impl TransactionGraph {
    pub fn from_dataset(dataset: &InvestigationDataset) -> Self {
        let mut accounts = BTreeSet::new();

        let mut edges = Vec::new();

        for transaction in &dataset.transactions {
            accounts.insert(transaction.sender_account.clone());
            accounts.insert(transaction.recipient_account.clone());

            edges.push(TransactionEdge {
                transaction_id: transaction.id,
                external_reference: transaction.external_reference.clone(),
                from_account: transaction.sender_account.clone(),
                to_account: transaction.recipient_account.clone(),
                amount_minor: transaction.amount_minor,
                currency: transaction.currency.clone(),
                transaction_type: transaction.transaction_type.clone(),
                occurred_at: transaction.occurred_at,
            });
        }

        let nodes = accounts
            .into_iter()
            .map(|account_id| AccountNode { account_id })
            .collect();

        Self {
            case_id: dataset.case.id,
            nodes,
            edges,
        }
    }

    pub fn build_relationships(&self) -> Vec<AccountRelationship> {
        let mut relationships: BTreeMap<(String, String), Vec<i64>> = BTreeMap::new();

        for edge in &self.edges {
            let key = (edge.from_account.clone(), edge.to_account.clone());

            relationships
                .entry(key)
                .or_default()
                .push(edge.transaction_id);
        }

        relationships
            .into_iter()
            .map(
                |((from_account, to_account), transaction_ids)| AccountRelationship {
                    from_account,
                    to_account,
                    transaction_count: transaction_ids.len(),
                    transaction_ids,
                },
            )
            .collect()
    }

    pub fn build_adjacency_list(&self) -> BTreeMap<String, Vec<String>> {
        let mut adjacency: BTreeMap<String, BTreeSet<String>> = self
            .nodes
            .iter()
            .map(|node| (node.account_id.clone(), BTreeSet::new()))
            .collect();

        for edge in &self.edges {
            adjacency
                .entry(edge.from_account.clone())
                .or_default()
                .insert(edge.to_account.clone());
        }

        adjacency
            .into_iter()
            .map(|(account, neighbors)| (account, neighbors.into_iter().collect()))
            .collect()
    }

    pub fn find_paths(&self, start_account: &str, max_hops: usize) -> Vec<TransactionPath> {
        let adjacency = self.build_adjacency_list();

        if max_hops == 0 || !adjacency.contains_key(start_account) {
            return Vec::new();
        }

        let mut results = Vec::new();

        let mut current_path = vec![start_account.to_owned()];

        let mut visited = BTreeSet::new();
        visited.insert(start_account.to_owned());

        Self::dfs_paths(
            start_account,
            max_hops,
            &adjacency,
            &mut current_path,
            &mut visited,
            &mut results,
        );

        results
    }

    fn dfs_paths(
        current_account: &str,
        max_hops: usize,
        adjacency: &BTreeMap<String, Vec<String>>,
        current_path: &mut Vec<String>,
        visited: &mut BTreeSet<String>,
        results: &mut Vec<TransactionPath>,
    ) {
        let current_hops = current_path.len() - 1;

        if current_hops >= max_hops {
            return;
        }

        let Some(neighbors) = adjacency.get(current_account) else {
            return;
        };

        for neighbor in neighbors {
            if visited.contains(neighbor) {
                continue;
            }

            current_path.push(neighbor.clone());
            visited.insert(neighbor.clone());

            results.push(TransactionPath {
                accounts: current_path.clone(),
                hops: current_path.len() - 1,
            });

            Self::dfs_paths(
                neighbor,
                max_hops,
                adjacency,
                current_path,
                visited,
                results,
            );

            visited.remove(neighbor);
            current_path.pop();
        }
    }
    pub fn to_response(&self) -> TransactionGraphResponse {
        TransactionGraphResponse {
            case_id: self.case_id,
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
            relationships: self.build_relationships(),
            adjacency_list: self.build_adjacency_list(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TransactionGraph;
    use chrono::Utc;
    use std::collections::BTreeSet;

    use crate::models::case_response::CaseResponse;
    use crate::models::investigation_dataset::{DatasetTransaction, InvestigationDataset};
    use crate::models::status::CaseStatus;

    fn sample_transaction(id: i64, sender: &str, recipient: &str) -> DatasetTransaction {
        let now = Utc::now();

        DatasetTransaction {
            id,
            external_reference: format!("TXN-{id}"),
            amount_minor: 5000,
            currency: String::from("INR"),
            sender_account: sender.to_owned(),
            recipient_account: recipient.to_owned(),
            transaction_type: String::from("transfer"),
            occurred_at: now,
            linked_at: now,
        }
    }

    fn sample_dataset() -> InvestigationDataset {
        let now = Utc::now();

        InvestigationDataset {
            case: CaseResponse {
                id: 42,
                title: String::from("Graph investigation"),
                risk_score: 50,
                status: CaseStatus::Open,
                created_at: now,
                updated_at: now,
            },
            investigator: None,
            transaction_count: 3,
            transactions: vec![
                sample_transaction(1, "ACCOUNT-A", "ACCOUNT-B"),
                sample_transaction(2, "ACCOUNT-A", "ACCOUNT-C"),
                sample_transaction(3, "ACCOUNT-B", "ACCOUNT-C"),
            ],
            evidence_count: 0,
            evidence: Vec::new(),
        }
    }

    #[test]
    fn builds_unique_account_nodes() {
        let dataset = sample_dataset();

        let graph = TransactionGraph::from_dataset(&dataset);

        assert_eq!(graph.nodes.len(), 3);

        let account_ids: Vec<&str> = graph
            .nodes
            .iter()
            .map(|node| node.account_id.as_str())
            .collect();

        assert_eq!(account_ids, vec!["ACCOUNT-A", "ACCOUNT-B", "ACCOUNT-C"]);
    }

    #[test]
    fn creates_one_edge_for_each_transaction() {
        let dataset = sample_dataset();

        let graph = TransactionGraph::from_dataset(&dataset);

        assert_eq!(graph.edges.len(), 3);
    }

    #[test]
    fn preserves_transaction_direction() {
        let dataset = sample_dataset();

        let graph = TransactionGraph::from_dataset(&dataset);

        let edge = &graph.edges[0];

        assert_eq!(edge.from_account, "ACCOUNT-A");
        assert_eq!(edge.to_account, "ACCOUNT-B");
        assert_eq!(edge.transaction_id, 1);
    }

    #[test]
    fn preserves_case_id() {
        let dataset = sample_dataset();

        let graph = TransactionGraph::from_dataset(&dataset);

        assert_eq!(graph.case_id, 42);
    }

    #[test]
    fn preserves_multiple_transactions_between_same_accounts() {
        let mut dataset = sample_dataset();

        dataset
            .transactions
            .push(sample_transaction(4, "ACCOUNT-A", "ACCOUNT-B"));

        let graph = TransactionGraph::from_dataset(&dataset);

        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 4);
    }

    #[test]
    fn empty_dataset_produces_empty_graph() {
        let mut dataset = sample_dataset();

        dataset.transactions.clear();
        dataset.transaction_count = 0;

        let graph = TransactionGraph::from_dataset(&dataset);

        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
    }
    #[test]
    fn groups_transactions_between_same_accounts() {
        let mut dataset = sample_dataset();

        dataset
            .transactions
            .push(sample_transaction(4, "ACCOUNT-A", "ACCOUNT-B"));

        let graph = TransactionGraph::from_dataset(&dataset);
        let relationships = graph.build_relationships();

        let relationship = relationships
            .iter()
            .find(|relationship| {
                relationship.from_account == "ACCOUNT-A" && relationship.to_account == "ACCOUNT-B"
            })
            .unwrap();

        assert_eq!(relationship.transaction_count, 2);
        assert_eq!(relationship.transaction_ids, vec![1, 4]);
    }

    #[test]
    fn keeps_opposite_directions_as_separate_relationships() {
        let mut dataset = sample_dataset();

        dataset
            .transactions
            .push(sample_transaction(4, "ACCOUNT-B", "ACCOUNT-A"));

        let graph = TransactionGraph::from_dataset(&dataset);
        let relationships = graph.build_relationships();

        let forward = relationships.iter().find(|relationship| {
            relationship.from_account == "ACCOUNT-A" && relationship.to_account == "ACCOUNT-B"
        });

        let reverse = relationships.iter().find(|relationship| {
            relationship.from_account == "ACCOUNT-B" && relationship.to_account == "ACCOUNT-A"
        });

        assert!(forward.is_some());
        assert!(reverse.is_some());

        assert_eq!(forward.unwrap().transaction_count, 1);
        assert_eq!(reverse.unwrap().transaction_count, 1);
    }

    #[test]
    fn produces_one_relationship_for_each_unique_directional_pair() {
        let dataset = sample_dataset();

        let graph = TransactionGraph::from_dataset(&dataset);
        let relationships = graph.build_relationships();

        assert_eq!(relationships.len(), 3);
    }

    #[test]
    fn preserves_transaction_ids_in_relationships() {
        let mut dataset = sample_dataset();

        dataset
            .transactions
            .push(sample_transaction(4, "ACCOUNT-A", "ACCOUNT-B"));

        dataset
            .transactions
            .push(sample_transaction(5, "ACCOUNT-A", "ACCOUNT-B"));

        let graph = TransactionGraph::from_dataset(&dataset);
        let relationships = graph.build_relationships();

        let relationship = relationships
            .iter()
            .find(|relationship| {
                relationship.from_account == "ACCOUNT-A" && relationship.to_account == "ACCOUNT-B"
            })
            .unwrap();

        assert_eq!(relationship.transaction_ids, vec![1, 4, 5]);
    }

    #[test]
    fn builds_correct_outgoing_adjacency_list() {
        let dataset = sample_dataset();

        let graph = TransactionGraph::from_dataset(&dataset);
        let adjacency = graph.build_adjacency_list();

        assert_eq!(
            adjacency.get("ACCOUNT-A").unwrap(),
            &vec![String::from("ACCOUNT-B"), String::from("ACCOUNT-C"),]
        );

        assert_eq!(
            adjacency.get("ACCOUNT-B").unwrap(),
            &vec![String::from("ACCOUNT-C")]
        );
    }

    #[test]
    fn does_not_duplicate_neighbors() {
        let mut dataset = sample_dataset();

        dataset
            .transactions
            .push(sample_transaction(4, "ACCOUNT-A", "ACCOUNT-B"));

        let graph = TransactionGraph::from_dataset(&dataset);
        let adjacency = graph.build_adjacency_list();

        let neighbors = adjacency.get("ACCOUNT-A").unwrap();

        assert_eq!(neighbors.len(), 2);
        assert_eq!(
            neighbors,
            &vec![String::from("ACCOUNT-B"), String::from("ACCOUNT-C"),]
        );
    }

    #[test]
    fn preserves_direction_of_connections() {
        let dataset = sample_dataset();

        let graph = TransactionGraph::from_dataset(&dataset);
        let adjacency = graph.build_adjacency_list();

        assert!(
            adjacency
                .get("ACCOUNT-A")
                .unwrap()
                .contains(&String::from("ACCOUNT-B"))
        );

        assert!(
            !adjacency
                .get("ACCOUNT-B")
                .unwrap()
                .contains(&String::from("ACCOUNT-A"))
        );
    }

    #[test]
    fn includes_accounts_without_outgoing_transactions() {
        let dataset = sample_dataset();

        let graph = TransactionGraph::from_dataset(&dataset);
        let adjacency = graph.build_adjacency_list();

        assert_eq!(adjacency.get("ACCOUNT-C").unwrap(), &Vec::<String>::new());
    }

    #[test]
    fn empty_graph_produces_empty_adjacency_list() {
        let mut dataset = sample_dataset();
        dataset.transactions.clear();

        let graph = TransactionGraph::from_dataset(&dataset);
        let adjacency = graph.build_adjacency_list();

        assert!(adjacency.is_empty());
    }

    #[test]
    fn finds_direct_transaction_paths() {
        let dataset = sample_dataset();

        let graph = TransactionGraph::from_dataset(&dataset);
        let paths = graph.find_paths("ACCOUNT-A", 1);

        assert_eq!(paths.len(), 2);

        assert!(paths.iter().any(|path| {
            path.accounts == vec![String::from("ACCOUNT-A"), String::from("ACCOUNT-B")]
        }));
    }

    #[test]
    fn finds_multi_hop_paths() {
        let dataset = sample_dataset();

        let graph = TransactionGraph::from_dataset(&dataset);
        let paths = graph.find_paths("ACCOUNT-A", 2);

        assert!(paths.iter().any(|path| {
            path.accounts
                == vec![
                    String::from("ACCOUNT-A"),
                    String::from("ACCOUNT-B"),
                    String::from("ACCOUNT-C"),
                ]
        }));

        assert!(paths.iter().any(|path| {
            path.accounts == vec![String::from("ACCOUNT-A"), String::from("ACCOUNT-C")]
        }));
    }

    #[test]
    fn respects_maximum_hops() {
        let dataset = sample_dataset();

        let graph = TransactionGraph::from_dataset(&dataset);
        let paths = graph.find_paths("ACCOUNT-A", 1);

        assert!(paths.iter().all(|path| path.hops <= 1));
    }

    #[test]
    fn returns_no_paths_for_unknown_account() {
        let dataset = sample_dataset();

        let graph = TransactionGraph::from_dataset(&dataset);
        let paths = graph.find_paths("UNKNOWN-ACCOUNT", 3);

        assert!(paths.is_empty());
    }

    #[test]
    fn returns_no_paths_when_max_hops_is_zero() {
        let dataset = sample_dataset();

        let graph = TransactionGraph::from_dataset(&dataset);
        let paths = graph.find_paths("ACCOUNT-A", 0);

        assert!(paths.is_empty());
    }

    #[test]
    fn does_not_revisit_accounts_in_a_path() {
        let mut dataset = sample_dataset();

        // Add a cycle: ACCOUNT-C -> ACCOUNT-A.
        dataset
            .transactions
            .push(sample_transaction(4, "ACCOUNT-C", "ACCOUNT-A"));

        let graph = TransactionGraph::from_dataset(&dataset);
        let paths = graph.find_paths("ACCOUNT-A", 5);

        for path in paths {
            let unique_accounts: BTreeSet<&String> = path.accounts.iter().collect();

            assert_eq!(unique_accounts.len(), path.accounts.len());
        }
    }
}
