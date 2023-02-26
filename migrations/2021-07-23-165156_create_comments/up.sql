CREATE TABLE comments (
    body text NOT NULL,
    coach_id uuid NOT NULL REFERENCES coaches(id),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    drawing text NOT NULL DEFAULT '',
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    review_id uuid NOT NULL REFERENCES reviews(id),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    video_timestamp INTEGER NOT NULL DEFAULT 0
);

SELECT diesel_manage_updated_at('comments');
