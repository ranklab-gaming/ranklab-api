ALTER TABLE recordings DROP COLUMN metadata;
ALTER TABLE comments DROP COLUMN metadata;
ALTER TABLE recordings ALTER COLUMN video_key SET NOT NULL;
ALTER TABLE comments ALTER COLUMN video_timestamp SET NOT NULL;
