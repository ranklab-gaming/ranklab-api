ALTER TABLE users RENAME TO players;
CREATE TABLE coaches (
    bio text NOT NULL,
    country text NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    email text UNIQUE NOT NULL,
    game_id text NOT NULL,
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL,
    password text,
    price integer NOT NULL,
    stripe_account_id text NOT NULL,
    stripe_details_submitted boolean NOT NULL DEFAULT false,
    stripe_payouts_enabled boolean NOT NULL DEFAULT false,
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    emails_enabled boolean NOT NULL DEFAULT true,
    slug text NOT NULL UNIQUE,
    avatar_id uuid REFERENCES avatars(id) ON DELETE SET NULL,
    approved boolean NOT NULL DEFAULT false,
    reviews_count INT NOT NULL DEFAULT 0
);
SELECT diesel_manage_updated_at('coaches');
CREATE TYPE review_state AS ENUM ('awaiting_payment', 'awaiting_review', 'draft', 'published', 'accepted', 'refunded');
CREATE TABLE reviews (
    coach_id uuid NOT NULL REFERENCES coaches(id),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    notes text NOT NULL DEFAULT '',
    player_id uuid NOT NULL REFERENCES players(id),
    recording_id uuid NOT NULL REFERENCES recordings(id),
    state review_state NOT NULL DEFAULT 'awaiting_payment',
    stripe_payment_intent_id text NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);
SELECT diesel_manage_updated_at('reviews');
ALTER TABLE one_time_tokens ADD COLUMN coach_id uuid REFERENCES coaches(id);
ALTER TABLE one_time_tokens RENAME COLUMN user_id TO player_id;
ALTER TABLE comments ADD COLUMN coach_id uuid REFERENCES coaches(id);
ALTER TABLE comments ADD COLUMN review_id uuid REFERENCES reviews(id);
ALTER TABLE comments DROP COLUMN user_id;
ALTER TABLE comments DROP COLUMN recording_id;
ALTER TABLE one_time_tokens DROP COLUMN scope;
ALTER TABLE audios ADD COLUMN review_id uuid REFERENCES reviews(id);
ALTER TABLE audios DROP COLUMN user_id;
ALTER TABLE avatars DROP COLUMN user_id;
DELETE FROM one_time_tokens;
DELETE FROM recordings;
DELETE FROM players;
ALTER TABLE players ADD COLUMN skill_level integer NOT NULL;
ALTER TABLE players ADD COLUMN stripe_customer_id text NOT NULL;
ALTER TABLE recordings DROP COLUMN notes;
ALTER TABLE recordings RENAME COLUMN user_id TO player_id;
ALTER INDEX users_pkey RENAME TO players_pkey;
ALTER INDEX users_email_key RENAME TO players_email_key;
