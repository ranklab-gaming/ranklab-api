CREATE TABLE digests (
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at timestamp NOT NULL DEFAULT now(),
    updated_at timestamp NOT NULL DEFAULT now(),
    metadata jsonb NOT NULL DEFAULT '{}'::jsonb
);

SELECT diesel_manage_updated_at('digests');
ALTER TABLE users DROP COLUMN digest_notified_at;
CREATE UNIQUE INDEX followings_game_id_user_id ON followings (game_id, user_id);
