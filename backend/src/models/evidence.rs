use chrono::{DateTime, Utc};

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct Evidence {
    pub id: i64,
    pub case_id: i64,
    pub title: String,
    pub description: String,
    pub evidence_type: String,
    pub source_reference: String,
    pub sha256_hash: Option<String>,
    pub collected_by: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, serde::Serialize)]
pub struct EvidencePackage {
    pub case_id: i64,
    pub generated_at: DateTime<Utc>,
    pub evidence_count: usize,
    pub evidence: Vec<Evidence>,
}

#[derive(Debug, Clone)]
pub struct NewEvidence {
    pub case_id: i64,
    pub title: String,
    pub description: String,
    pub evidence_type: String,
    pub source_reference: String,
    pub sha256_hash: Option<String>,
    pub collected_by: i64,
}

impl NewEvidence {
    pub fn normalize(mut self) -> Self {
        self.title = self.title.trim().to_owned();
        self.description = self.description.trim().to_owned();
        self.evidence_type = self.evidence_type.trim().to_ascii_lowercase();
        self.source_reference = self.source_reference.trim().to_owned();

        self.sha256_hash = self
            .sha256_hash
            .map(|hash| hash.trim().to_ascii_lowercase());

        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.case_id <= 0 {
            return Err(String::from("Case ID must be greater than zero"));
        }

        if self.collected_by <= 0 {
            return Err(String::from("Collector ID must be greater than zero"));
        }

        if self.title.is_empty() {
            return Err(String::from("Evidence title cannot be empty"));
        }

        if self.description.is_empty() {
            return Err(String::from("Evidence description cannot be empty"));
        }

        if self.source_reference.is_empty() {
            return Err(String::from("Evidence source reference cannot be empty"));
        }

        if ![
            "bank_statement",
            "transaction_record",
            "account_activity",
            "document",
            "other",
        ]
        .contains(&self.evidence_type.as_str())
        {
            return Err(String::from("Unsupported evidence type"));
        }

        if let Some(hash) = &self.sha256_hash {
            if hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err(String::from(
                    "SHA-256 hash must contain 64 hexadecimal characters",
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::NewEvidence;

    fn sample_evidence() -> NewEvidence {
        NewEvidence {
            case_id: 42,
            title: String::from("Bank statement"),
            description: String::from("Statement showing suspicious transfers"),
            evidence_type: String::from("bank_statement"),
            source_reference: String::from("storage://case-42/statement.pdf"),
            sha256_hash: None,
            collected_by: 10,
        }
    }

    #[test]
    fn accepts_valid_evidence() {
        assert!(sample_evidence().normalize().validate().is_ok());
    }

    #[test]
    fn rejects_invalid_case_or_collector_ids() {
        let mut evidence = sample_evidence();
        evidence.case_id = 0;
        assert!(evidence.normalize().validate().is_err());

        let mut evidence = sample_evidence();
        evidence.collected_by = 0;
        assert!(evidence.normalize().validate().is_err());
    }

    #[test]
    fn rejects_unsupported_evidence_types() {
        let mut evidence = sample_evidence();
        evidence.evidence_type = String::from("unknown");

        assert!(evidence.normalize().validate().is_err());
    }

    #[test]
    fn rejects_invalid_sha256_hashes() {
        let mut evidence = sample_evidence();
        evidence.sha256_hash = Some(String::from("not-a-valid-hash"));

        assert!(evidence.normalize().validate().is_err());
    }

    #[test]
    fn normalizes_evidence_fields() {
        let mut evidence = sample_evidence();
        evidence.title = String::from("  Bank statement  ");
        evidence.evidence_type = String::from(" BANK_STATEMENT ");
        evidence.sha256_hash = Some("A".repeat(64));

        let evidence = evidence.normalize();

        assert_eq!(evidence.title, "Bank statement");
        assert_eq!(evidence.evidence_type, "bank_statement");
        assert_eq!(
            evidence.sha256_hash.as_deref(),
            Some("a".repeat(64).as_str())
        );
        assert!(evidence.validate().is_ok());
    }
}
