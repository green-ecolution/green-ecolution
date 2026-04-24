-- +goose Up
ALTER TABLE vehicles ADD COLUMN archived_at TIMESTAMP DEFAULT NULL;

-- +goose Down
ALTER TABLE vehicles DROP COLUMN archived_at;
