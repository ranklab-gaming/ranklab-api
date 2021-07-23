CREATE TABLE users (
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    auth0_id text NOT NULL
);
