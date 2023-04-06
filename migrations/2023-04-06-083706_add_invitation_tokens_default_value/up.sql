ALTER TABLE coach_invitations ALTER COLUMN value SET DEFAULT MD5(random()::text);
