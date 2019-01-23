CREATE TABLE problem_statements (
	id BIGSERIAL PRIMARY KEY,
	problem_id BIGINT NOT NULL REFERENCES problems,
	filename VARCHAR(200) NOT NULL,
	mimetype VARCHAR(100) NOT NULL,
	statement BYTEA NOT NULL,
	created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX problem_statements_problem ON problem_statements (problem_id);

CREATE TRIGGER update_problem_statements_timestamp BEFORE UPDATE ON problem_statements FOR EACH ROW EXECUTE PROCEDURE update_timestamp();
