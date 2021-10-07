CREATE TABLE reviews (
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id),
    coach_id uuid REFERENCES coaches(id),
    title text NOT NULL DEFAULT '',
    video_url text NOT NULL,
    game_id uuid NOT NULL REFERENCES games(id)
);
