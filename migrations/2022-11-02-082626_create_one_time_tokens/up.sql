CREATE TABLE one_time_tokens (
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    value TEXT NOT NULL,
    player_id uuid REFERENCES players(id),
    coach_id uuid REFERENCES coaches(id),
    scope TEXT NOT NULL,
    used_at TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

SELECT diesel_manage_updated_at('one_time_tokens');
