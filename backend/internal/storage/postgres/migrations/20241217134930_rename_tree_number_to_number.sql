-- +goose Up
-- +goose StatementBegin
ALTER TABLE trees
RENAME COLUMN tree_number TO number;
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
ALTER TABLE trees
RENAME COLUMN number TO tree_number;
-- +goose StatementEnd
