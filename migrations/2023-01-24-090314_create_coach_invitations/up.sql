CREATE TABLE coach_invitations (
    id uuid NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    value TEXT NOT NULL,
    used_at TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

SELECT diesel_manage_updated_at('coach_invitations');
