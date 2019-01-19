CREATE TABLE contestproblems (
	contest_id BIGINT NOT NULL REFERENCES contests,
	problem_id BIGINT NOT NULL REFERENCES problems,
	label VARCHAR(200) NOT NULL,

	PRIMARY KEY(contest_id, problem_id)
);
