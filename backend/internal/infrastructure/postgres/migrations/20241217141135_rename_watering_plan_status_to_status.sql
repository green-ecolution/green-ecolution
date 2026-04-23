-- +goose Up
-- +goose StatementBegin
ALTER TABLE watering_plans
RENAME COLUMN watering_plan_status TO status;
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
ALTER TABLE watering_plans
RENAME COLUMN status TO watering_plan_status;
-- +goose StatementEnd
