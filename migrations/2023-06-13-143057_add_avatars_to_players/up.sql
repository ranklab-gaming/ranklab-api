ALTER TABLE players ADD COLUMN avatar_id uuid REFERENCES avatars(id) ON DELETE SET NULL;
ALTER TABLE avatars DROP COLUMN coach_id;
