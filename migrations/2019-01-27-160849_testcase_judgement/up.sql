CREATE TABLE test_case_judgements (
	judgement_id BIGINT NOT NULL,
	judgement_grader_id INT NOT NULL,
	test_case_position INT NOT NULL,
	status INT NOT NULL,
	output BYTEA NOT NULL,
	error BYTEA NOT NULL,
	created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

	PRIMARY KEY (judgement_grader_id, judgement_id, test_case_position),
	FOREIGN KEY (judgement_grader_id, judgement_id) REFERENCES judgements (grader_id, id)
);
