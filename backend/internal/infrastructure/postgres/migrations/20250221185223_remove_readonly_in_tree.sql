-- +goose Up
ALTER TABLE trees DROP COLUMN readonly;

-- +goose Down
ALTER TABLE trees ADD COLUMN readonly BOOLEAN NOT NULL DEFAULT FALSE;
