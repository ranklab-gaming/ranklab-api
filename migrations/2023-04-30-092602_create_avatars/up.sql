CREATE TYPE avatar_state AS ENUM ('created', 'uploaded', 'processed');

CREATE TABLE avatars (
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    image_key text NOT NULL,
    processed_image_key text,
    state avatar_state NOT NULL DEFAULT 'created',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

SELECT diesel_manage_updated_at('avatars');

ALTER TABLE coaches ADD COLUMN avatar_id UUID REFERENCES avatars(id) ON DELETE SET NULL;
