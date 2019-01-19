CREATE TABLE contests (
	id BIGSERIAL PRIMARY KEY,
	name VARCHAR(200) NOT NULL,
	short_name VARCHAR(200) NOT NULL,
	start_at TIMESTAMP WITH TIME ZONE,
	freeze_at TIMESTAMP WITH TIME ZONE,
	end_at TIMESTAMP WITH TIME ZONE,
	created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TRIGGER update_contests_timestamp BEFORE UPDATE ON contests FOR EACH ROW EXECUTE PROCEDURE update_timestamp();

CREATE UNIQUE INDEX contests_short_name ON contests ((lower(short_name)));
