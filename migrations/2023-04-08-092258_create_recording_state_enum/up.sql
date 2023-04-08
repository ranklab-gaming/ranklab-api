CREATE TYPE recording_state AS ENUM ('created', 'uploaded', 'processed');

ALTER TABLE recordings ADD COLUMN state recording_state NOT NULL DEFAULT 'created';
ALTER TABLE recordings ADD COLUMN thumbnail_key TEXT;
ALTER TABLE recordings ADD COLUMN processed_video_key TEXT;

UPDATE recordings SET state = 'uploaded' WHERE uploaded IS TRUE;

ALTER TABLE recordings DROP COLUMN uploaded;
