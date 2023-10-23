CREATE TABLE followings (
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL references users(id),
    game_id text NOT NULL,
    created_at timestamp NOT NULL DEFAULT now(),
    updated_at timestamp NOT NULL DEFAULT now()
);

SELECT diesel_manage_updated_at('followings');

INSERT INTO followings (user_id, game_id)
SELECT id, 'overwatch' FROM users WHERE created_at < '2023-10-18';

INSERT INTO followings (user_id, game_id)
SELECT id, 'apex' FROM users WHERE created_at >= '2023-10-18';
