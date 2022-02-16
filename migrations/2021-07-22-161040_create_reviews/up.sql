CREATE TABLE reviews (
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    player_id uuid NOT NULL REFERENCES players(id),
    coach_id uuid REFERENCES coaches(id),
    title text NOT NULL DEFAULT '',
    recording_id uuid NOT NULL REFERENCES recordings(id),
    game_id text NOT NULL DEFAULT '',
    skill_level smallint NOT NULL DEFAULT 0,
    notes text NOT NULL DEFAULT '',
    published boolean NOT NULL DEFAULT false
)
