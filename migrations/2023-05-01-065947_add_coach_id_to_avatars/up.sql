ALTER TABLE avatars ADD COLUMN coach_id uuid NOT NULL REFERENCES coaches(id);
