CREATE TABLE recordings (
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    mime_type text NOT NULL,
    player_id uuid NOT NULL REFERENCES players(id),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    uploaded boolean NOT NULL DEFAULT false,
    video_key text NOT NULL
);

SELECT diesel_manage_updated_at('recordings');
