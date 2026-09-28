ALTER TABLE cases
ADD COLUMN status TEXT NOT NULL DEFAULT 'OPEN'
CHECK (
    status IN (
        'OPEN',
        'UNDER_REVIEW',
        'ESCALATED',
        'CLOSED',
        'ARCHIVED'
    )
);
