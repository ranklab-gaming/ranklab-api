ALTER TABLE players DROP COLUMN avatar_id;
ALTER TABLE avatars ADD COLUMN coach_id uuid references coaches(id);
