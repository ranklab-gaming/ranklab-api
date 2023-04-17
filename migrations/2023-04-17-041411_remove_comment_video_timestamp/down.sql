-- Create the video_timestamp column
ALTER TABLE comments
ADD COLUMN video_timestamp INTEGER;

-- Update the video_timestamp column with the data from the metadata column
UPDATE comments
SET video_timestamp = CAST((metadata->'video'->>'timestamp') AS INTEGER)
WHERE metadata->'video'->>'timestamp' IS NOT NULL;

-- Remove the video timestamp data from the metadata column
UPDATE comments
SET metadata = metadata #- '{video}'
WHERE metadata->'video'->>'timestamp' IS NOT NULL;

ALTER TABLE comments
ALTER COLUMN metadata DROP NOT NULL;
