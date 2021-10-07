CREATE TABLE coaches (
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users(id),
    name text NOT NULL DEFAULT '',
    email text UNIQUE NOT NULL DEFAULT '',
    bio text NOT NULL DEFAULT '',
    game_id uuid NOT NULL REFERENCES games(id)
);
