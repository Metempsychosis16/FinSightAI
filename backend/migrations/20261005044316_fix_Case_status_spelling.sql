ALTER TABLE cases
DROP CONSTRAINT cases_status_check;

ALTER TABLE cases
ADD CONSTRAINT cases_status_check
CHECK (
    status IN (
        'OPEN',
        'UNDER_REVIEW',
        'ESCALATED',
        'CLOSED',
        'ARCHIVED'
    )
);
