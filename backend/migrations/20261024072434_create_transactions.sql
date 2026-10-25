CREATE TABLE transactions (
    id BIGSERIAL PRIMARY KEY,

    external_reference TEXT NOT NULL UNIQUE
        CHECK (LENGTH(BTRIM(external_reference)) > 0),

    amount_minor BIGINT NOT NULL
        CHECK (amount_minor > 0),

    currency TEXT NOT NULL
        CHECK (currency ~ '^[A-Z]{3}$'),

    sender_account TEXT NOT NULL
        CHECK (LENGTH(BTRIM(sender_account)) > 0),

    recipient_account TEXT NOT NULL
        CHECK (LENGTH(BTRIM(recipient_account)) > 0),

    transaction_type TEXT NOT NULL
        CHECK (
            transaction_type IN (
                'transfer',
                'deposit',
                'withdrawal',
                'payment'
            )
        ),

    occurred_at TIMESTAMPTZ NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX transactions_occurred_at_idx
    ON transactions (occurred_at DESC);

CREATE INDEX transactions_sender_account_idx
    ON transactions (sender_account);

CREATE INDEX transactions_recipient_account_idx
    ON transactions (recipient_account);
