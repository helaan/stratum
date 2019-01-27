CREATE TABLE judgements (
	id BIGSERIAL NOT NULL,
	grader_id INT NOT NULL,
	submission_id BIGINT NOT NULL,
	submission_location_id INT NOT NULL,
	status INT NOT NULL,
	score BIGINT,
	valid BOOLEAN NOT NULL,
	created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,

	PRIMARY KEY (grader_id, id),
	FOREIGN KEY (submission_location_id, submission_id) REFERENCES submissions (location_id, id)
);

CREATE INDEX judgements_submission_fk ON judgements (submission_location_id, submission_id);
CREATE TRIGGER update_judgements_timestamp BEFORE UPDATE ON judgements FOR EACH ROW EXECUTE PROCEDURE update_timestamp();
