CREATE TYPE media_state AS ENUM ('created', 'uploaded', 'processed');

ALTER TABLE
  avatars
ADD
  COLUMN new_state media_state NOT NULL DEFAULT 'created';

ALTER TABLE
  recordings
ADD
  COLUMN new_state media_state NOT NULL DEFAULT 'created';

UPDATE
  avatars
SET
  new_state = state :: text :: media_state;

UPDATE
  recordings
SET
  new_state = state :: text :: media_state;

ALTER TABLE
  avatars DROP COLUMN state;

ALTER TABLE
  recordings DROP COLUMN state;

ALTER TABLE
  avatars RENAME COLUMN new_state TO state;

ALTER TABLE
  recordings RENAME COLUMN new_state TO state;

DROP TYPE avatar_state;

DROP TYPE recording_state;

CREATE TABLE audios (
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  review_id uuid NOT NULL REFERENCES reviews(id),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
  audio_key text NOT NULL,
  processed_audio_key text,
  state media_state NOT NULL DEFAULT 'created'
);

SELECT
  diesel_manage_updated_at('audios');

ALTER TABLE
  comments
ADD
  COLUMN audio_id uuid REFERENCES audios(id);
