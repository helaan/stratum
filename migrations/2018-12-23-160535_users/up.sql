CREATE TABLE users (
	id BIGSERIAL PRIMARY KEY,
	-- Users are part of a single team
	team_id BIGINT DEFAULT NULL REFERENCES teams(id) ON DELETE SET NULL,
	username VARCHAR(200) UNIQUE NOT NULL,
	password_hash VARCHAR(200) NOT NULL,
	rights SMALLINT DEFAULT 1,
	created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TRIGGER update_users_timestamp BEFORE UPDATE ON users FOR EACH ROW EXECUTE PROCEDURE update_timestamp();
