-- +goose Up
-- +goose StatementBegin
ALTER TABLE watering_plans
ADD COLUMN gpx_url TEXT;
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
ALTER TABLE watering_plans
DROP COLUMN gpx_url;
-- +goose StatementEnd
