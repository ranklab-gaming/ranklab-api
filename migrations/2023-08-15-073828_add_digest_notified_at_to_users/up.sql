-- Your SQL goes here
ALTER TABLE users
ADD COLUMN digest_notified_at TIMESTAMP NOT NULL DEFAULT NOW();
