CREATE TYPE avatar_state AS ENUM ('created', 'uploaded', 'processed');

CREATE TYPE recording_state AS ENUM ('created', 'uploaded', 'processed');

ALTER TABLE
  avatars
ADD
  COLUMN old_state avatar_state NOT NULL DEFAULT 'created';

ALTER TABLE
  recordings
ADD
  COLUMN old_state recording_state NOT NULL DEFAULT 'created';

UPDATE
  avatars
SET
  old_state = state :: text :: avatar_state;

UPDATE
  recordings
SET
  old_state = state :: text :: recording_state;

ALTER TABLE
  avatars DROP COLUMN state;

ALTER TABLE
  recordings DROP COLUMN state;

ALTER TABLE
  avatars RENAME COLUMN old_state TO state;

ALTER TABLE
  recordings RENAME COLUMN old_state TO state;

UPDATE
  comments
SET
  audio_id = NULL;

ALTER TABLE
  comments DROP COLUMN audio_id;

DROP TABLE audios;

DROP TYPE media_state;
