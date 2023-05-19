CREATE TABLE coach_invitations (
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    used_at TIMESTAMP,
    value TEXT NOT NULL
);

SELECT diesel_manage_updated_at('coach_invitations');
