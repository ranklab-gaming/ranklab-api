CREATE TABLE one_time_tokens (
    coach_id uuid REFERENCES coaches(id),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id uuid REFERENCES players(id),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    used_at TIMESTAMP,
    value TEXT NOT NULL
);

SELECT diesel_manage_updated_at('one_time_tokens');
