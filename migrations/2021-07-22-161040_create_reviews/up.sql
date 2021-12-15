CREATE TABLE reviews (
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id),
    coach_id uuid REFERENCES coaches(id),
    title text NOT NULL DEFAULT '',
    recording_id uuid NOT NULL REFERENCES recordings(id),
    game_id text NOT NULL DEFAULT '',
    notes text NOT NULL DEFAULT ''
);
