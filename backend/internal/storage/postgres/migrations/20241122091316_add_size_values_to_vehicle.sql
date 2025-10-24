-- +goose Up
-- +goose StatementBegin
CREATE TYPE driving_license AS ENUM ('B', 'BE', 'C');

ALTER TABLE vehicles
ADD COLUMN model TEXT NOT NULL,
ADD COLUMN driving_license driving_license NOT NULL DEFAULT 'B',
ADD COLUMN height FLOAT NOT NULL DEFAULT 0,
ADD COLUMN length FLOAT NOT NULL DEFAULT 0,
ADD COLUMN width FLOAT NOT NULL DEFAULT 0;
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
ALTER TABLE vehicles
DROP COLUMN model,
DROP COLUMN driving_license,
DROP COLUMN height,
DROP COLUMN length,
DROP COLUMN width;

DROP TYPE IF EXISTS driving_license;
-- +goose StatementEnd
