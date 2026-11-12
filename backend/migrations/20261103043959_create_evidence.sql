CREATE TABLE evidence (
    id BIGSERIAL PRIMARY KEY,

    case_id BIGINT NOT NULL
        REFERENCES cases(id) ON DELETE RESTRICT,

    title TEXT NOT NULL
        CHECK (LENGTH(BTRIM(title)) > 0),

    description TEXT NOT NULL
        CHECK (LENGTH(BTRIM(description)) > 0),

    evidence_type TEXT NOT NULL
        CHECK (
            evidence_type IN (
                'bank_statement',
                'transaction_record',
                'account_activity',
                'document',
                'other'
            )
        ),

    source_reference TEXT NOT NULL
        CHECK (LENGTH(BTRIM(source_reference)) > 0),

    sha256_hash TEXT
        CHECK (
            sha256_hash IS NULL
            OR sha256_hash ~ '^[a-fA-F0-9]{64}$'
        ),

    collected_by BIGINT NOT NULL
        REFERENCES users(id) ON DELETE RESTRICT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX evidence_case_id_idx
    ON evidence (case_id);

CREATE INDEX evidence_collected_by_idx
    ON evidence (collected_by);
