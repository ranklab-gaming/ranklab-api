CREATE TABLE recordings (
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id),
    extension text NOT NULL DEFAULT '',
    uploaded boolean NOT NULL DEFAULT false
);
