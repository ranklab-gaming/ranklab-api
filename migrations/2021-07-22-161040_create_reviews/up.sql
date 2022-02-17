CREATE TYPE review_state AS ENUM ('awaiting_payment', 'awaiting_review', 'draft', 'published', 'refunded');

CREATE TABLE reviews (
    coach_id uuid REFERENCES coaches(id),
    game_id text NOT NULL DEFAULT '',
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    notes text NOT NULL DEFAULT '',
    player_id uuid NOT NULL REFERENCES players(id),
    recording_id uuid NOT NULL REFERENCES recordings(id),
    skill_level smallint NOT NULL DEFAULT 0,
    title text NOT NULL DEFAULT '',
    state review_state NOT NULL DEFAULT 'awaiting_payment',
    stripe_payment_intent_id text NOT NULL DEFAULT ''
);

CREATE INDEX reviews_recording_id_idx ON reviews (recording_id);
