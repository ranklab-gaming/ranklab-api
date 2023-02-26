CREATE TABLE coaches (
    bio text NOT NULL,
    country text NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    email text UNIQUE NOT NULL,
    game_ids text[] NOT NULL DEFAULT array[]::text[],
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL,
    password text NOT NULL,
    stripe_account_id text NOT NULL,
    stripe_details_submitted boolean NOT NULL DEFAULT false,
    stripe_payouts_enabled boolean NOT NULL DEFAULT false,
    stripe_product_id text NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

SELECT diesel_manage_updated_at('coaches');
