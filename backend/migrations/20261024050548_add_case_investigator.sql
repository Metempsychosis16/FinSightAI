ALTER TABLE cases
ADD COLUMN investigator_id BIGINT
REFERENCES investigators(id)
ON DELETE RESTRICT;

CREATE INDEX cases_investigator_id_idx
ON cases(investigator_id);
