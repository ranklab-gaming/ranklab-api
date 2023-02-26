CREATE TABLE players (
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    email text UNIQUE NOT NULL,
    games jsonb[] NOT NULL DEFAULT '{}',
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL,
    password text NOT NULL,
    stripe_customer_id text,
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

SELECT diesel_manage_updated_at('players');
