-- +goose Up
-- +goose StatementBegin
ALTER TABLE tree_cluster_watering_plans
ADD COLUMN consumed_water FLOAT NOT NULL DEFAULT 0.0;
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
ALTER TABLE tree_cluster_watering_plans
DROP COLUMN consumed_water;
-- +goose StatementEnd
