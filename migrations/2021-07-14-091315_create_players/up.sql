CREATE TABLE players (
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    auth0_id text NOT NULL,
    name text NOT NULL DEFAULT '',
    email text UNIQUE NOT NULL DEFAULT '',
    games jsonb[] NOT NULL DEFAULT '{}',
    stripe_customer_id text,
    can_create_reviews boolean NOT NULL DEFAULT false
);
