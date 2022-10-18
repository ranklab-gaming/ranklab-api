CREATE TABLE players (
    email text UNIQUE NOT NULL DEFAULT '',
    name text NOT NULL DEFAULT '',
    games jsonb[] NOT NULL DEFAULT '{}',
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    stripe_customer_id text,
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

SELECT diesel_manage_updated_at('players');
