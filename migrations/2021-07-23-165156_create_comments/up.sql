CREATE TABLE comments (
    body text NOT NULL DEFAULT '',
    coach_id uuid NOT NULL REFERENCES coaches(id),
    drawing text NOT NULL DEFAULT '',
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    review_id uuid NOT NULL REFERENCES reviews(id),
    video_timestamp INTEGER NOT NULL DEFAULT 0,
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

SELECT diesel_manage_updated_at('comments');
