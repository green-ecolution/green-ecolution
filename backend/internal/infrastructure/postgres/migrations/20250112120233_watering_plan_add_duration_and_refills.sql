-- +goose Up
-- +goose StatementBegin
ALTER TABLE watering_plans
ADD COLUMN refill_count INT NOT NULL DEFAULT 0;

ALTER TABLE watering_plans
ADD COLUMN duration FLOAT NOT NULL DEFAULT 0;
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
ALTER TABLE watering_plans
DROP COLUMN refill_count;

ALTER TABLE watering_plans
DROP COLUMN duration;
-- +goose StatementEnd

