CREATE TABLE coaches (
    email text UNIQUE NOT NULL DEFAULT '',
    name text NOT NULL DEFAULT '',
    bio text NOT NULL DEFAULT '',
    country text NOT NULL DEFAULT '',
    game_ids text[] NOT NULL DEFAULT array[]::text[],
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    stripe_account_id text,
    stripe_details_submitted boolean NOT NULL DEFAULT false,
    stripe_payouts_enabled boolean NOT NULL DEFAULT false,
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

SELECT diesel_manage_updated_at('coaches');
