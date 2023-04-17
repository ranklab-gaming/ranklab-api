-- Update the metadata column with the video timestamp
UPDATE comments
SET metadata =
    CASE
        WHEN metadata IS NULL THEN
            jsonb_build_object('video', jsonb_build_object('timestamp', video_timestamp))
        ELSE
            COALESCE(metadata, '{}'::jsonb) || jsonb_build_object('video', jsonb_build_object('timestamp', video_timestamp))
    END
WHERE video_timestamp IS NOT NULL;

-- Remove the video_timestamp column
ALTER TABLE comments
DROP COLUMN video_timestamp;

ALTER TABLE comments
ALTER COLUMN metadata SET NOT NULL;
