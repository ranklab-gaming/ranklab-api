CREATE TABLE comments (
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    review_id uuid NOT NULL REFERENCES reviews(id),
    coach_id uuid NOT NULL REFERENCES coaches(id),
    body text NOT NULL DEFAULT '',
    video_timestamp INTEGER NOT NULL DEFAULT 0,
    drawing text NOT NULL DEFAULT ''
);
