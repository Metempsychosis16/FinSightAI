use sqlx::PgPool;

use crate::models::case_response::CaseRow;
use crate::models::evidence::Evidence;
use crate::models::investigation_dataset::DatasetTransaction;
use crate::models::investigator::Investigator;

pub struct InvestigationDatasetRows {
    pub case: CaseRow,
    pub investigator: Option<Investigator>,
    pub transactions: Vec<DatasetTransaction>,
    pub evidence: Vec<Evidence>,
}

pub struct InvestigationDatasetRepository<'a> {
    db: &'a PgPool,
}

impl<'a> InvestigationDatasetRepository<'a> {
    pub fn new(db: &'a PgPool) -> Self {
        Self { db }
    }

    pub async fn load(
        &self,
        case_id: i64,
    ) -> Result<Option<InvestigationDatasetRows>, sqlx::Error> {
        let mut tx = self.db.begin().await?;

        // Use a consistent PostgreSQL snapshot across all reads.
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
            .execute(&mut *tx)
            .await?;

        // 1. Load the requested case.
        let case = sqlx::query_as::<_, CaseRow>(
            r#"
            SELECT
                id,
                title,
                risk_score,
                status,
                created_at,
                updated_at
            FROM cases
            WHERE id = $1
            "#,
        )
        .bind(case_id)
        .fetch_optional(&mut *tx)
        .await?;

        let Some(case) = case else {
            return Ok(None);
        };

        // 2. Load the assigned investigator, if any.
        let investigator = sqlx::query_as::<_, Investigator>(
            r#"
            SELECT
                i.id,
                i.user_id,
                i.name,
                i.department,
                i.years_experience,
                i.created_at,
                i.updated_at
            FROM cases AS c
            JOIN investigators AS i
                ON i.id = c.investigator_id
            WHERE c.id = $1
            "#,
        )
        .bind(case_id)
        .fetch_optional(&mut *tx)
        .await?;

        // 3. Load transactions linked to this case.
        let transactions = sqlx::query_as::<_, DatasetTransaction>(
            r#"
            SELECT
                t.id,
                t.external_reference,
                t.amount_minor,
                t.currency,
                t.sender_account,
                t.recipient_account,
                t.transaction_type,
                t.occurred_at,
                ct.linked_at
            FROM case_transactions AS ct
            JOIN transactions AS t
                ON t.id = ct.transaction_id
            WHERE ct.case_id = $1
            ORDER BY t.occurred_at ASC, t.id ASC
            "#,
        )
        .bind(case_id)
        .fetch_all(&mut *tx)
        .await?;

        // 4. Load evidence metadata for this case.
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
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(Some(InvestigationDatasetRows {
            case,
            investigator,
            transactions,
            evidence,
        }))
    }
}
