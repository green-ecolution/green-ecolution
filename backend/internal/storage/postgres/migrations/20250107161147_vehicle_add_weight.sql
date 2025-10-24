-- +goose Up
-- +goose StatementBegin
ALTER TABLE vehicles
ADD COLUMN weight FLOAT NOT NULL DEFAULT 0;
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
ALTER TABLE vehicles
DROP COLUMN weight
-- +goose StatementEnd
