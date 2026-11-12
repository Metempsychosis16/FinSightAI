
CREATE TABLE case_transactions (
    case_id BIGINT NOT NULL
        REFERENCES cases(id) ON DELETE RESTRICT,

    transaction_id BIGINT NOT NULL
        REFERENCES transactions(id) ON DELETE RESTRICT,

    linked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (case_id, transaction_id)
);

CREATE INDEX case_transactions_transaction_id_idx
    ON case_transactions (transaction_id);

