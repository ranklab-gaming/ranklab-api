ALTER TABLE comments DROP CONSTRAINT comments_recording_id_fkey;
ALTER TABLE comments ADD CONSTRAINT comments_recording_id_fkey FOREIGN KEY (recording_id) REFERENCES recordings(id) ON DELETE CASCADE;
