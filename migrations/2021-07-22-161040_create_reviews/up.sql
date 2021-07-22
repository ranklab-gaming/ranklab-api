CREATE TABLE reviews (
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id),
    coach_id uuid REFERENCES coaches(id),
    title character VARYING NOT NULL,
    video_url character VARYING NOT NULL,
    game character VARYING NOT NULL
);
