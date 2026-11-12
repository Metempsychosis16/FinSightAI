use sqlx::PgPool;

use crate::models::evidence::{Evidence, NewEvidence};

#[derive(Debug)]
pub enum EvidenceRepositoryError {
    Database(sqlx::Error),
    CaseNotFound,
}

pub struct EvidenceRepository<'a> {
    db: &'a PgPool,
}

impl<'a> EvidenceRepository<'a> {
    pub fn new(db: &'a PgPool) -> Self {
        Self { db }
    }

    pub async fn create(
        &self,
        evidence: &NewEvidence,
    ) -> Result<Evidence, EvidenceRepositoryError> {
        sqlx::query_as::<_, Evidence>(
            r#"
            INSERT INTO evidence (
                case_id,
                title,
                description,
                evidence_type,
                source_reference,
                sha256_hash,
                collected_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING
                id,
                case_id,
                title,
                description,
                evidence_type,
                source_reference,
                sha256_hash,
                collected_by,
                created_at
            "#,
        )
        .bind(evidence.case_id)
        .bind(&evidence.title)
        .bind(&evidence.description)
        .bind(&evidence.evidence_type)
        .bind(&evidence.source_reference)
        .bind(&evidence.sha256_hash)
        .bind(evidence.collected_by)
        .fetch_one(self.db)
        .await
        .map_err(EvidenceRepositoryError::Database)
    }

    pub async fn list_by_case(
        &self,
        case_id: i64,
    ) -> Result<Vec<Evidence>, EvidenceRepositoryError> {
        let case_exists =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM cases WHERE id = $1)")
                .bind(case_id)
                .fetch_one(self.db)
                .await
                .map_err(EvidenceRepositoryError::Database)?;

        if !case_exists {
            return Err(EvidenceRepositoryError::CaseNotFound);
        }

        let evidence = sqlx::query_as::<_, Evidence>(
            r#"
        SELECT
            id,
            case_id,
            title,
            description,
            evidence_type,
            source_reference,
            sha256_hash,
            collected_by,
            created_at
        FROM evidence
        WHERE case_id = $1
        ORDER BY created_at ASC, id ASC
        "#,
        )
        .bind(case_id)
        .fetch_all(self.db)
        .await
        .map_err(EvidenceRepositoryError::Database)?;

        Ok(evidence)
    }
}
