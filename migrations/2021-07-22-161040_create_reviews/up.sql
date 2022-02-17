CREATE TABLE reviews (
    coach_id uuid REFERENCES coaches(id),
    game_id text NOT NULL DEFAULT '',
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    notes text NOT NULL DEFAULT '',
    player_id uuid NOT NULL REFERENCES players(id),
    published boolean NOT NULL DEFAULT false,
    recording_id uuid NOT NULL REFERENCES recordings(id),
    skill_level smallint NOT NULL DEFAULT 0,
    title text NOT NULL DEFAULT ''
)
