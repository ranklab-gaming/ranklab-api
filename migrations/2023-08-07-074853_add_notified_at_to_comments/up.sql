ALTER TABLE comments
ADD COLUMN notified_at TIMESTAMP;

UPDATE comments
SET notified_at = NOW();
