
CREATE TABLE investigators (
    id BIGSERIAL PRIMARY KEY,

    user_id BIGINT NOT NULL UNIQUE
        REFERENCES users(id) ON DELETE RESTRICT,

    name TEXT NOT NULL
        CHECK (LENGTH(BTRIM(name)) > 0),

    department TEXT NOT NULL
        CHECK (LENGTH(BTRIM(department)) > 0),

    years_experience SMALLINT NOT NULL
        CHECK (years_experience BETWEEN 0 AND 59),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

