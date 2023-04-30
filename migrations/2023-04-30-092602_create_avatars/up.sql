CREATE TYPE avatar_state AS ENUM ('created', 'uploaded', 'processed');

CREATE TABLE avatars (
    id UUID PRIMARY KEY,
    image_key text NOT NULL,
    processed_image_key text NULL,
    state avatar_state NOT NULL DEFAULT 'created',
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

SELECT diesel_manage_updated_at('avatars');

ALTER TABLE coaches ADD COLUMN avatar_id UUID REFERENCES avatars(id);
