
CREATE TABLE ai_investigation_reports (
    id BIGSERIAL PRIMARY KEY,

    case_id BIGINT NOT NULL
        REFERENCES cases(id) ON DELETE RESTRICT,

    generated_by BIGINT NOT NULL
        REFERENCES users(id) ON DELETE RESTRICT,

    provider TEXT NOT NULL
        CHECK (LENGTH(BTRIM(provider)) > 0),

    model TEXT NOT NULL
        CHECK (LENGTH(BTRIM(model)) > 0),

    prompt_version TEXT NOT NULL
        CHECK (LENGTH(BTRIM(prompt_version)) > 0),

    schema_version TEXT NOT NULL
        CHECK (LENGTH(BTRIM(schema_version)) > 0),

    structured_report JSONB NOT NULL
        CHECK (jsonb_typeof(structured_report) = 'object'),

    markdown_report TEXT NOT NULL
        CHECK (LENGTH(BTRIM(markdown_report)) > 0),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX ai_investigation_reports_case_created_idx
    ON ai_investigation_reports (case_id, created_at DESC, id DESC);

CREATE INDEX ai_investigation_reports_generated_by_idx
    ON ai_investigation_reports (generated_by);

