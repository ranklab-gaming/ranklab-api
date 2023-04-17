-- Create the drawing column
ALTER TABLE comments
ADD COLUMN drawing TEXT;

-- Update the drawing column with the data from the metadata column
UPDATE comments
SET drawing = CAST((metadata->'video'->>'drawing') AS TEXT)
WHERE metadata->'video'->>'drawing' IS NOT NULL;

-- Remove the drawing data from the metadata column
UPDATE comments
SET metadata = metadata #- '{video, drawing}'
WHERE metadata->'video'->>'drawing' IS NOT NULL;
