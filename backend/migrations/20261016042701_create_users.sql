CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,

    email TEXT NOT NULL
        CHECK (email = LOWER(BTRIM(email))),

    password_hash TEXT NOT NULL
        CHECK (LENGTH(password_hash) > 0),

    role TEXT NOT NULL DEFAULT 'INVESTIGATOR'
        CHECK (
            role IN (
                'INVESTIGATOR',
                'SENIOR_INVESTIGATOR',
                'ADMIN'
            )
        ),

    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX users_email_unique_idx
ON users (email);
