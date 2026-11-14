use std::collections::HashMap;

use crate::models::investigation_dataset::InvestigationDataset;
use crate::risk::models::{
    RiskAssessment, RiskExplanation, RiskFactorExplanation, RiskIndicator, RiskLevel,
};

#[derive(Debug, Clone)]
pub struct RiskRuleConfig {
    pub high_value_enabled: bool,
    pub high_value_currency: String,
    pub high_value_threshold_minor: i64,
    pub high_value_points: u8,

    pub high_volume_enabled: bool,
    pub high_volume_threshold: usize,
    pub high_volume_points: u8,

    pub repeated_recipient_enabled: bool,
    pub repeated_recipient_threshold: usize,
    pub repeated_recipient_points: u8,

    pub missing_evidence_hash_enabled: bool,
    pub missing_evidence_hash_points: u8,
}

impl Default for RiskRuleConfig {
    fn default() -> Self {
        Self {
            high_value_enabled: true,
            high_value_currency: String::from("INR"),
            high_value_threshold_minor: 1_000_000,
            high_value_points: 25,

            high_volume_enabled: true,
            high_volume_threshold: 10,
            high_volume_points: 15,

            repeated_recipient_enabled: true,
            repeated_recipient_threshold: 3,
            repeated_recipient_points: 20,

            missing_evidence_hash_enabled: true,
            missing_evidence_hash_points: 10,
        }
    }
}

pub trait RiskEngine {
    fn assess(&self, dataset: &InvestigationDataset) -> RiskAssessment;
}

pub struct FinancialCrimeRiskEngine {
    config: RiskRuleConfig,
}

impl FinancialCrimeRiskEngine {
    pub fn new() -> Self {
        Self {
            config: RiskRuleConfig::default(),
        }
    }

    pub fn with_config(config: RiskRuleConfig) -> Self {
        Self { config }
    }

    fn detect_indicators(&self, dataset: &InvestigationDataset) -> Vec<RiskIndicator> {
        let mut indicators = Vec::new();

        if self.config.high_value_enabled
            && dataset.transactions.iter().any(|transaction| {
                transaction.currency == self.config.high_value_currency
                    && transaction.amount_minor >= self.config.high_value_threshold_minor
            })
        {
            indicators.push(RiskIndicator {
                code: String::from("HIGH_VALUE_TRANSACTION"),
                description: format!(
                    "At least one {} transaction meets or exceeds the configured high-value threshold",
                    self.config.high_value_currency
                ),
                points: self.config.high_value_points,
            });
        }

        let transaction_count = dataset.transactions.len();

        if self.config.high_volume_enabled
            && dataset.transaction_count >= self.config.high_volume_threshold
        {
            indicators.push(RiskIndicator {
                code: String::from("HIGH_TRANSACTION_VOLUME"),
                description: format!("The case contains {transaction_count} linked transactions",),
                points: self.config.high_volume_points,
            });
        }

        if self.config.repeated_recipient_enabled {
            let mut recipient_counts: HashMap<&str, usize> = HashMap::new();
            for transaction in &dataset.transactions {
                *recipient_counts
                    .entry(transaction.recipient_account.as_str())
                    .or_insert(0) += 1;
            }

            if recipient_counts
                .values()
                .any(|count| *count >= self.config.repeated_recipient_threshold)
            {
                indicators.push(RiskIndicator {
                    code: String::from("REPEATED_RECIPIENT"),
                    description: format!(
                        "At least one recipient received {} or more linked transactions",
                        self.config.repeated_recipient_threshold
                    ),
                    points: self.config.repeated_recipient_points,
                });
            }
        }

        if self.config.missing_evidence_hash_enabled
            && dataset
                .evidence
                .iter()
                .any(|evidence| evidence.sha256_hash.is_none())
        {
            indicators.push(RiskIndicator {
                code: String::from("MISSING_EVIDENCE_HASH"),
                description: String::from(
                    "At least one evidence record has no recorded SHA-256 hash",
                ),
                points: self.config.missing_evidence_hash_points,
            });
        }

        indicators
    }
}

fn calculate_raw_points(indicators: &[RiskIndicator]) -> u16 {
    indicators
        .iter()
        .map(|indicator| u16::from(indicator.points))
        .sum()
}

fn calculate_score(raw_points: u16) -> u8 {
    raw_points.min(100) as u8
}

fn classify_risk(score: u8) -> RiskLevel {
    match score {
        0..=19 => RiskLevel::Low,
        20..=39 => RiskLevel::Medium,
        40..=59 => RiskLevel::High,
        _ => RiskLevel::Critical,
    }
}

fn build_explanation(
    score: u8,
    raw_points: u16,
    level: RiskLevel,
    indicators: &[RiskIndicator],
) -> RiskExplanation {
    let level_name = match level {
        RiskLevel::Low => "LOW",
        RiskLevel::Medium => "MEDIUM",
        RiskLevel::High => "HIGH",
        RiskLevel::Critical => "CRITICAL",
    };

    let summary = if indicators.is_empty() {
        String::from("No configured risk indicators were detected. The score is 0.")
    } else if raw_points > 100 {
        format!(
            "The case received a capped risk score of {score}/100 ({level_name}). The raw indicator total was {raw_points} points across {} detected indicators.",
            indicators.len()
        )
    } else {
        format!(
            "The case received a risk score of {score}/100 ({level_name}) based on {} detected risk indicator(s).",
            indicators.len()
        )
    };

    let factors = indicators
        .iter()
        .map(|indicator| RiskFactorExplanation {
            code: indicator.code.clone(),
            description: indicator.description.clone(),
            points: indicator.points,
        })
        .collect();

    RiskExplanation {
        summary,
        raw_points,
        factors,
    }
}

impl RiskEngine for FinancialCrimeRiskEngine {
    fn assess(&self, dataset: &InvestigationDataset) -> RiskAssessment {
        let indicators = self.detect_indicators(dataset);
        let raw_points = calculate_raw_points(&indicators);
        let score = calculate_score(raw_points);

        let level = classify_risk(score);

        let explanation = build_explanation(score, raw_points, level, &indicators);

        RiskAssessment {
            case_id: dataset.case.id,
            score,
            level,
            indicators,
            explanation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    use crate::models::case_response::CaseResponse;
    use crate::models::evidence::Evidence;
    use crate::models::investigation_dataset::{DatasetTransaction, InvestigationDataset};
    use crate::models::status::CaseStatus;

    const HIGH_VALUE_THRESHOLD_MINOR: i64 = 1_000_000;
    const HIGH_TRANSACTION_VOLUME_THRESHOLD: usize = 10;
    const REPEATED_RECIPIENT_THRESHOLD: usize = 3;

    fn sample_dataset() -> InvestigationDataset {
        let now = Utc::now();

        InvestigationDataset {
            case: CaseResponse {
                id: 42,
                title: String::from("Suspicious transfers"),
                risk_score: 85,
                status: CaseStatus::Open,
                created_at: now,
                updated_at: now,
            },
            investigator: None,
            transaction_count: 0,
            transactions: Vec::new(),
            evidence_count: 0,
            evidence: Vec::new(),
        }
    }

    fn sample_transaction(id: i64, amount_minor: i64, recipient: &str) -> DatasetTransaction {
        let now = Utc::now();

        DatasetTransaction {
            id,
            external_reference: format!("TXN-{id}"),
            amount_minor,
            currency: String::from("INR"),
            sender_account: String::from("ACCOUNT-A"),
            recipient_account: recipient.to_owned(),
            transaction_type: String::from("transfer"),
            occurred_at: now,
            linked_at: now,
        }
    }

    fn sample_evidence(hash: Option<String>) -> Evidence {
        Evidence {
            id: 1,
            case_id: 42,
            title: String::from("Bank statement"),
            description: String::from("Statement for investigation"),
            evidence_type: String::from("bank_statement"),
            source_reference: String::from("storage://case-42/statement.pdf"),
            sha256_hash: hash,
            collected_by: 10,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn detects_high_value_transaction() {
        let mut dataset = sample_dataset();

        dataset.transactions.push(sample_transaction(
            1,
            HIGH_VALUE_THRESHOLD_MINOR,
            "ACCOUNT-B",
        ));

        let engine = FinancialCrimeRiskEngine::new();
        let result = engine.assess(&dataset);

        assert!(
            result
                .indicators
                .iter()
                .any(|indicator| indicator.code == "HIGH_VALUE_TRANSACTION")
        );
    }

    #[test]
    fn detects_high_transaction_volume() {
        let mut dataset = sample_dataset();
        dataset.transaction_count = HIGH_TRANSACTION_VOLUME_THRESHOLD;

        for id in 1..=HIGH_TRANSACTION_VOLUME_THRESHOLD as i64 {
            dataset
                .transactions
                .push(sample_transaction(id, 5000, "ACCOUNT-B"));
        }

        let engine = FinancialCrimeRiskEngine::new();
        let result = engine.assess(&dataset);

        assert!(
            result
                .indicators
                .iter()
                .any(|indicator| indicator.code == "HIGH_TRANSACTION_VOLUME")
        );
    }

    #[test]
    fn detects_repeated_recipient() {
        let mut dataset = sample_dataset();

        for id in 1..=REPEATED_RECIPIENT_THRESHOLD as i64 {
            dataset
                .transactions
                .push(sample_transaction(id, 5000, "ACCOUNT-B"));
        }

        let engine = FinancialCrimeRiskEngine::new();
        let result = engine.assess(&dataset);

        assert!(
            result
                .indicators
                .iter()
                .any(|indicator| indicator.code == "REPEATED_RECIPIENT")
        );
    }

    #[test]
    fn detects_evidence_without_hash() {
        let mut dataset = sample_dataset();
        dataset.evidence.push(sample_evidence(None));

        let engine = FinancialCrimeRiskEngine::new();
        let result = engine.assess(&dataset);

        assert!(
            result
                .indicators
                .iter()
                .any(|indicator| indicator.code == "MISSING_EVIDENCE_HASH")
        );
    }

    #[test]
    fn does_not_flag_evidence_with_hash() {
        let mut dataset = sample_dataset();
        dataset.evidence.push(sample_evidence(Some("a".repeat(64))));

        let engine = FinancialCrimeRiskEngine::new();
        let result = engine.assess(&dataset);

        assert!(
            !result
                .indicators
                .iter()
                .any(|indicator| indicator.code == "MISSING_EVIDENCE_HASH")
        );
    }

    #[test]
    fn returns_no_indicators_for_empty_dataset() {
        let dataset = sample_dataset();

        let engine = FinancialCrimeRiskEngine::new();
        let result = engine.assess(&dataset);

        assert!(result.indicators.is_empty());
    }

    #[test]
    fn calculates_score_and_preserves_case_id() {
        let mut dataset = sample_dataset();

        dataset.transactions.push(sample_transaction(
            1,
            HIGH_VALUE_THRESHOLD_MINOR,
            "ACCOUNT-B",
        ));

        let engine = FinancialCrimeRiskEngine::new();
        let result = engine.assess(&dataset);

        assert_eq!(result.case_id, 42);
        assert_eq!(result.score, 25);
        assert_eq!(result.level, RiskLevel::Medium);
        assert_eq!(result.indicators.len(), 1);
        assert_eq!(result.indicators[0].points, 25);
    }

    #[test]
    fn classifies_all_risk_levels_correctly() {
        assert_eq!(classify_risk(0), RiskLevel::Low);
        assert_eq!(classify_risk(19), RiskLevel::Low);

        assert_eq!(classify_risk(20), RiskLevel::Medium);
        assert_eq!(classify_risk(39), RiskLevel::Medium);

        assert_eq!(classify_risk(40), RiskLevel::High);
        assert_eq!(classify_risk(59), RiskLevel::High);

        assert_eq!(classify_risk(60), RiskLevel::Critical);
        assert_eq!(classify_risk(100), RiskLevel::Critical);
    }

    #[test]
    fn calculates_total_indicator_points() {
        let indicators = vec![
            RiskIndicator {
                code: String::from("RULE_A"),
                description: String::from("Test indicator A"),
                points: 25,
            },
            RiskIndicator {
                code: String::from("RULE_B"),
                description: String::from("Test indicator B"),
                points: 15,
            },
            RiskIndicator {
                code: String::from("RULE_C"),
                description: String::from("Test indicator C"),
                points: 20,
            },
        ];

        let raw_points = calculate_raw_points(&indicators);

        assert_eq!(calculate_score(raw_points), 60);
    }

    #[test]
    fn caps_risk_score_at_100() {
        let indicators = vec![
            RiskIndicator {
                code: String::from("RULE_A"),
                description: String::from("Test indicator A"),
                points: 80,
            },
            RiskIndicator {
                code: String::from("RULE_B"),
                description: String::from("Test indicator B"),
                points: 80,
            },
        ];

        let raw_points = calculate_raw_points(&indicators);
        assert_eq!(calculate_score(raw_points), 100);
    }

    #[test]
    fn empty_indicators_produce_zero_score() {
        let indicators: Vec<RiskIndicator> = Vec::new();

        let raw_points = calculate_raw_points(&indicators);
        assert_eq!(calculate_score(raw_points), 0);
    }

    #[test]
    fn explains_why_a_risk_score_was_assigned() {
        let mut dataset = sample_dataset();

        dataset.transactions.push(sample_transaction(
            1,
            HIGH_VALUE_THRESHOLD_MINOR,
            "ACCOUNT-B",
        ));

        let engine = FinancialCrimeRiskEngine::new();
        let result = engine.assess(&dataset);

        assert_eq!(result.score, 25);
        assert_eq!(result.explanation.factors.len(), 1);

        assert_eq!(result.explanation.factors[0].code, "HIGH_VALUE_TRANSACTION");

        assert_eq!(result.explanation.factors[0].points, 25);

        assert!(result.explanation.summary.contains("risk score of 25/100"));
    }

    #[test]
    fn explanation_contains_every_detected_indicator() {
        let mut dataset = sample_dataset();

        dataset.transactions.push(sample_transaction(
            1,
            HIGH_VALUE_THRESHOLD_MINOR,
            "ACCOUNT-B",
        ));

        dataset
            .transactions
            .push(sample_transaction(2, 5000, "ACCOUNT-B"));

        dataset
            .transactions
            .push(sample_transaction(3, 5000, "ACCOUNT-B"));

        let engine = FinancialCrimeRiskEngine::new();
        let result = engine.assess(&dataset);

        assert_eq!(result.explanation.factors.len(), result.indicators.len());

        for indicator in &result.indicators {
            assert!(
                result
                    .explanation
                    .factors
                    .iter()
                    .any(|factor| factor.code == indicator.code)
            );
        }
    }

    #[test]
    fn explains_when_no_risk_indicators_are_detected() {
        let dataset = sample_dataset();

        let engine = FinancialCrimeRiskEngine::new();
        let result = engine.assess(&dataset);

        assert_eq!(result.score, 0);
        assert!(result.explanation.factors.is_empty());

        assert!(
            result
                .explanation
                .summary
                .contains("No configured risk indicators")
        );
    }
    #[test]
    fn custom_high_value_threshold_changes_detection() {
        let mut dataset = sample_dataset();

        dataset
            .transactions
            .push(sample_transaction(1, 500_000, "ACCOUNT-B"));

        let config = RiskRuleConfig {
            high_value_threshold_minor: 400_000,
            ..RiskRuleConfig::default()
        };

        let engine = FinancialCrimeRiskEngine::with_config(config);
        let result = engine.assess(&dataset);

        assert!(
            result
                .indicators
                .iter()
                .any(|indicator| indicator.code == "HIGH_VALUE_TRANSACTION")
        );
    }

    #[test]
    fn disabled_rule_does_not_generate_indicator() {
        let mut dataset = sample_dataset();

        dataset.transactions.push(sample_transaction(
            1,
            HIGH_VALUE_THRESHOLD_MINOR,
            "ACCOUNT-B",
        ));

        let config = RiskRuleConfig {
            high_value_enabled: false,
            ..RiskRuleConfig::default()
        };

        let engine = FinancialCrimeRiskEngine::with_config(config);
        let result = engine.assess(&dataset);

        assert!(
            !result
                .indicators
                .iter()
                .any(|indicator| indicator.code == "HIGH_VALUE_TRANSACTION")
        );
    }

    #[test]
    fn custom_rule_points_change_final_score() {
        let mut dataset = sample_dataset();

        dataset.transactions.push(sample_transaction(
            1,
            HIGH_VALUE_THRESHOLD_MINOR,
            "ACCOUNT-B",
        ));

        let config = RiskRuleConfig {
            high_value_points: 40,
            ..RiskRuleConfig::default()
        };

        let engine = FinancialCrimeRiskEngine::with_config(config);
        let result = engine.assess(&dataset);

        assert_eq!(result.score, 40);
        assert_eq!(result.level, RiskLevel::High);
        assert_eq!(result.indicators[0].points, 40);
        assert_eq!(result.explanation.factors[0].points, 40);
    }

    #[test]
    fn custom_currency_changes_high_value_rule() {
        let mut dataset = sample_dataset();

        let mut transaction = sample_transaction(1, 500_000, "ACCOUNT-B");

        transaction.currency = String::from("USD");
        dataset.transactions.push(transaction);

        let config = RiskRuleConfig {
            high_value_currency: String::from("USD"),
            high_value_threshold_minor: 400_000,
            ..RiskRuleConfig::default()
        };

        let engine = FinancialCrimeRiskEngine::with_config(config);
        let result = engine.assess(&dataset);

        assert!(
            result
                .indicators
                .iter()
                .any(|indicator| indicator.code == "HIGH_VALUE_TRANSACTION")
        );
    }

    #[test]
    fn score_is_capped_but_raw_points_are_preserved() {
        let indicators = vec![
            RiskIndicator {
                code: String::from("RULE_A"),
                description: String::from("First rule"),
                points: 80,
            },
            RiskIndicator {
                code: String::from("RULE_B"),
                description: String::from("Second rule"),
                points: 80,
            },
        ];

        let raw_points = calculate_raw_points(&indicators);
        let score = calculate_score(raw_points);

        assert_eq!(raw_points, 160);
        assert_eq!(score, 100);
        assert_eq!(classify_risk(score), RiskLevel::Critical);
    }

    #[test]
    fn zero_transactions_do_not_trigger_volume_rule() {
        let dataset = sample_dataset();
        let engine = FinancialCrimeRiskEngine::new();

        let result = engine.assess(&dataset);

        assert!(
            !result
                .indicators
                .iter()
                .any(|indicator| indicator.code == "HIGH_TRANSACTION_VOLUME")
        );
    }

    #[test]
    fn explanation_preserves_raw_points() {
        let indicators = vec![
            RiskIndicator {
                code: String::from("RULE_A"),
                description: String::from("First rule"),
                points: 80,
            },
            RiskIndicator {
                code: String::from("RULE_B"),
                description: String::from("Second rule"),
                points: 80,
            },
        ];

        let explanation = build_explanation(100, 160, RiskLevel::Critical, &indicators);

        assert_eq!(explanation.raw_points, 160);
        assert_eq!(explanation.factors.len(), 2);
        assert!(explanation.summary.contains("raw indicator total was 160"));
    }
}
