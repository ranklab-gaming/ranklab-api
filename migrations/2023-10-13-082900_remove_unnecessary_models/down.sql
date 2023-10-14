CREATE TABLE audios (
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
  audio_key text NOT NULL,
  processed_audio_key text,
  state media_state NOT NULL DEFAULT 'created',
  transcript text,
  user_id uuid NOT NULL REFERENCES users(id)
);

SELECT diesel_manage_updated_at('audios');

ALTER TABLE comments ADD COLUMN audio_id uuid REFERENCES audios(id);
ALTER TABLE users ADD COLUMN game_id text NOT NULL DEFAULT 'overwatch';
ALTER TABLE users ADD COLUMN skill_level integer NOT NULL DEFAULT 0;
ALTER TABLE recordings ADD COLUMN metadata jsonb;
