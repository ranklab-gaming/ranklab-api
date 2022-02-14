CREATE TABLE recordings (
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id uuid NOT NULL REFERENCES players(id),
    video_key text NOT NULL DEFAULT '',
    upload_url text NOT NULL DEFAULT '',
    uploaded boolean NOT NULL DEFAULT false,
    mime_type text NOT NULL DEFAULT '',
    stripe_payment_intent_id text
);
