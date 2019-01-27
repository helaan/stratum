CREATE TABLE submissions (
	id BIGSERIAL NOT NULL,
	location_id INT NOT NULL,
	problem_id BIGINT NOT NULL REFERENCES problems,
	team_id BIGINT NOT NULL REFERENCES teams,
	entry_point VARCHAR NOT NULL,
	created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

	PRIMARY KEY(location_id, id)
);

CREATE INDEX submissions_problem_id ON submissions (problem_id);
CREATE INDEX submissions_team_id ON submissions (team_id);

CREATE TRIGGER update_submissions_timestamp BEFORE UPDATE ON submissions FOR EACH ROW EXECUTE PROCEDURE update_timestamp();
