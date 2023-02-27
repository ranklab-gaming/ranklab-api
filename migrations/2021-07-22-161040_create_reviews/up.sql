CREATE TYPE review_state AS ENUM ('awaiting_payment', 'awaiting_review', 'draft', 'published', 'accepted', 'refunded');

CREATE TABLE reviews (
    coach_id uuid NOT NULL REFERENCES coaches(id),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    game_id text NOT NULL,
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    notes text NOT NULL DEFAULT '',
    player_id uuid NOT NULL REFERENCES players(id),
    recording_id uuid NOT NULL REFERENCES recordings(id),
    skill_level smallint NOT NULL,
    state review_state NOT NULL DEFAULT 'awaiting_payment',
    stripe_payment_intent_id text NOT NULL,
    title text NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

SELECT diesel_manage_updated_at('reviews');
