-- +goose Up
-- +goose StatementBegin
ALTER TABLE trees
ADD COLUMN readonly BOOLEAN NOT NULL DEFAULT FALSE,
DROP height_above_sea_level,
DROP age;

-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
ALTER TABLE trees
DROP COLUMN readonly,
ADD height_above_sea_level FLOAT NOT NULL DEFAULT 0.0,
ADD age INT NOT NULL DEFAULT 0;

-- +goose StatementEnd
