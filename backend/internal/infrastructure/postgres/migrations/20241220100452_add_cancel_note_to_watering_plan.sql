-- +goose Up
-- +goose StatementBegin
ALTER TABLE watering_plans
ADD COLUMN cancellation_note TEXT NOT NULL DEFAULT '';
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
ALTER TABLE watering_plans
DROP COLUMN cancellation_note;
-- +goose StatementEnd
