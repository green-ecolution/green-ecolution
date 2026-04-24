-- +goose Up
-- +goose StatementBegin
ALTER TABLE trees ADD COLUMN last_watered TIMESTAMP;
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
ALTER TABLE trees DROP COLUMN last_watered;
-- +goose StatementEnd
