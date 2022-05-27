CREATE TYPE review_state AS ENUM ('awaiting_payment', 'awaiting_review', 'draft', 'published', 'accepted', 'refunded');

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
    stripe_order_id text NOT NULL DEFAULT '',
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

SELECT diesel_manage_updated_at('reviews');
CREATE INDEX reviews_recording_id_idx ON reviews (recording_id);
