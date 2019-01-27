CREATE TABLE submission_files (
	submission_id BIGSERIAL NOT NULL,
	submission_location_id INT NOT NULL,
	filename VARCHAR NOT NULL,
	mimetype VARCHAR NOT NULL,
	content BYTEA NOT NULL,

	PRIMARY KEY (submission_location_id, submission_id, filename),
	FOREIGN KEY (submission_location_id, submission_id) REFERENCES submissions (location_id, id)
);
