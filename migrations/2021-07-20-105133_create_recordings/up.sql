CREATE TABLE recordings (
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id),
    video_key text NOT NULL DEFAULT '',
    upload_url text NOT NULL DEFAULT '',
    uploaded boolean NOT NULL DEFAULT false,
    mime_type text NOT NULL DEFAULT ''
);
