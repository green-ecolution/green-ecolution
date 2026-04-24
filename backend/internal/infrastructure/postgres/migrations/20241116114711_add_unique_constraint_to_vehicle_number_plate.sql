-- +goose Up
-- +goose StatementBegin
ALTER TABLE vehicles
    ADD CONSTRAINT unique_number_plate UNIQUE (number_plate);
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
ALTER TABLE vehicles
    DROP CONSTRAINT unique_number_plate;
-- +goose StatementEnd
