ALTER TABLE recordings DROP COLUMN state;
ALTER TABLE recordings ADD COLUMN uploaded BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE recordings DROP COLUMN thumbnail_key;
ALTER TABLE recordings DROP COLUMN processed_video_key;

DROP TYPE recording_state;
