ALTER TABLE coaches ADD COLUMN slug text;
UPDATE coaches SET slug = LOWER(REPLACE(name, ' ', '-'));
ALTER TABLE coaches ALTER COLUMN slug SET NOT NULL;
ALTER TABLE coaches ADD UNIQUE (slug);
