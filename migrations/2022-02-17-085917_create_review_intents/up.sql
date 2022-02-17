CREATE TABLE review_intents (
    game_id text NOT NULL DEFAULT '',
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    notes text NOT NULL DEFAULT '',
    player_id uuid NOT NULL REFERENCES players(id),
    recording_id uuid REFERENCES recordings(id),
    review_id uuid REFERENCES reviews(id),
    stripe_payment_intent_id text NOT NULL DEFAULT '',
    title text NOT NULL DEFAULT ''
);

CREATE UNIQUE INDEX review_intents_recording_id_idx ON review_intents (recording_id);
