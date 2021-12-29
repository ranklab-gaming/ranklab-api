CREATE TABLE coaches (
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL DEFAULT '',
    email text UNIQUE NOT NULL DEFAULT '',
    bio text NOT NULL DEFAULT '',
    game_id text NOT NULL DEFAULT '',
    auth0_id text NOT NULL
);
