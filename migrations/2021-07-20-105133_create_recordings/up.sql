CREATE TABLE recordings (
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    mime_type text NOT NULL DEFAULT '',
    player_id uuid NOT NULL REFERENCES players(id),
    upload_url text NOT NULL DEFAULT '',
    uploaded boolean NOT NULL DEFAULT false,
    video_key text NOT NULL DEFAULT ''
);
