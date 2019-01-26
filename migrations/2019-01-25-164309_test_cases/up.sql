CREATE TABLE test_cases (
	problem_id BIGINT NOT NULL REFERENCES problems,
	position INT NOT NULL,
	description VARCHAR NOT NULL DEFAULT '',
	input BYTEA NOT NULL,
	input_mimetype VARCHAR NOT NULL DEFAULT 'text/plain',
	output BYTEA NOT NULL,
	output_mimetype VARCHAR NOT NULL DEFAULT 'text/plain',
	visible_rights SMALLINT NOT NULL DEFAULT 1000,
	created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

	PRIMARY KEY(problem_id, position)
);

CREATE TRIGGER update_test_cases_timestamp BEFORE UPDATE ON test_cases FOR EACH ROW EXECUTE PROCEDURE update_timestamp();
