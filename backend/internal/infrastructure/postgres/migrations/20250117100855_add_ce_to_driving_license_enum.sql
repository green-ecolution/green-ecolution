-- +goose Up
-- +goose StatementBegin
ALTER TYPE driving_license ADD VALUE 'CE';
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
-- Rename the current type to avoid conflicts
ALTER TYPE driving_license RENAME TO driving_license_old;

-- Create a new type without the 'CE' value
CREATE TYPE driving_license AS ENUM ('B', 'BE', 'C');

-- Update the column to ensure it doesn't have 'CE' or any non-valid values
UPDATE vehicles SET driving_license = 'B' WHERE driving_license = 'CE';

-- Remove the default value temporarily (this will make the type change work)
ALTER TABLE vehicles ALTER COLUMN driving_license DROP DEFAULT;

-- Alter the column to use the new type
ALTER TABLE vehicles 
    ALTER COLUMN driving_license TYPE driving_license USING driving_license::text::driving_license;

-- Set the default value back to 'B'
ALTER TABLE vehicles ALTER COLUMN driving_license SET DEFAULT 'B';

-- Drop the old type
DROP TYPE driving_license_old;
-- +goose StatementEnd
