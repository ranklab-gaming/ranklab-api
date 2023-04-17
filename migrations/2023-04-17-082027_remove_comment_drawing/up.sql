-- Update the metadata column with the drawing
UPDATE comments
SET metadata =
    CASE
        WHEN metadata IS NULL THEN
            jsonb_build_object('video', jsonb_build_object('drawing', drawing))
        ELSE
            COALESCE(metadata, '{}'::jsonb) || jsonb_build_object('video', jsonb_build_object('drawing', drawing))
    END
WHERE drawing <> '';


-- Remove the drawing column
ALTER TABLE comments
DROP COLUMN drawing;
