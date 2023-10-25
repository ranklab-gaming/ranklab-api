DROP TABLE digests;
ALTER TABLE users ADD COLUMN digest_notified_at timestamp NOT NULL DEFAULT now();
