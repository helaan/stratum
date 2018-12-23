-- Create a procedure to update the updated_at timestamp.
-- From: https://stackoverflow.com/questions/9556474/#9556527
CREATE OR REPLACE FUNCTION update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';
