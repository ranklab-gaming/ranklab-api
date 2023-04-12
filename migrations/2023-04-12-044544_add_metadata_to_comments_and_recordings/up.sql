ALTER TABLE recordings ADD COLUMN metadata jsonb;
ALTER TABLE comments ADD COLUMN metadata jsonb;
ALTER TABLE recordings ALTER COLUMN video_key DROP NOT NULL;
ALTER TABLE comments ALTER COLUMN video_timestamp DROP NOT NULL;
