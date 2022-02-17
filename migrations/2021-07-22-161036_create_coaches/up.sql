CREATE TABLE coaches (
    auth0_id text NOT NULL,
    bio text NOT NULL DEFAULT '',
    country text NOT NULL DEFAULT '',
    email text UNIQUE NOT NULL DEFAULT '',
    games jsonb[] NOT NULL DEFAULT '{}',
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL DEFAULT '',
    stripe_account_id text,
    stripe_details_submitted boolean NOT NULL DEFAULT false,
    stripe_payouts_enabled boolean NOT NULL DEFAULT false
);
