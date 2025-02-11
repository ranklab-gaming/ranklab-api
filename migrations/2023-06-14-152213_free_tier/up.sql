ALTER TABLE players RENAME TO users;
ALTER TABLE one_time_tokens DROP COLUMN coach_id;
ALTER TABLE one_time_tokens RENAME COLUMN player_id TO user_id;
DELETE FROM comments;
ALTER TABLE comments DROP COLUMN coach_id;
ALTER TABLE comments ADD COLUMN user_id uuid NOT NULL REFERENCES users(id);
ALTER TABLE comments DROP COLUMN review_id;
ALTER TABLE comments ADD COLUMN recording_id uuid NOT NULL REFERENCES recordings(id);
ALTER TABLE one_time_tokens ADD COLUMN scope TEXT;
UPDATE one_time_tokens SET scope = 'reset_password';
ALTER TABLE one_time_tokens ALTER COLUMN scope SET NOT NULL;
DELETE FROM audios;
ALTER TABLE audios DROP COLUMN review_id;
ALTER TABLE audios ADD COLUMN user_id uuid NOT NULL REFERENCES users(id);
UPDATE coaches SET avatar_id = NULL;
DELETE FROM avatars;
ALTER TABLE avatars ADD COLUMN user_id uuid NOT NULL REFERENCES users(id);
ALTER TABLE users DROP COLUMN skill_level;
ALTER TABLE users DROP COLUMN stripe_customer_id;
ALTER TABLE recordings ADD COLUMN notes text NOT NULL DEFAULT '';
ALTER TABLE recordings RENAME COLUMN player_id TO user_id;
ALTER TABLE reviews DROP COLUMN coach_id;
ALTER INDEX players_pkey RENAME TO users_pkey;
ALTER INDEX players_email_key RENAME TO users_email_key;
DROP TABLE coaches;
DROP TABLE reviews;
DROP TYPE review_state;
