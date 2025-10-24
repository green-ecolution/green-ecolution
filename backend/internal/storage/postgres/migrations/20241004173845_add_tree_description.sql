-- +goose Up
-- +goose StatementBegin
ALTER TABLE trees
ADD COLUMN description TEXT;
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
ALTER TABLE trees
DROP COLUMN description;
-- +goose StatementEnd
