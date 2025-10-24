-- +goose Up
-- +goose StatementBegin
ALTER TABLE sensor_data
    DROP CONSTRAINT IF EXISTS sensor_data_sensor_id_fkey,
    ADD CONSTRAINT sensor_data_sensor_id_fkey
    FOREIGN KEY (sensor_id) REFERENCES sensors(id) ON DELETE CASCADE;
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
ALTER TABLE sensor_data
    DROP CONSTRAINT IF EXISTS sensor_data_sensor_id_fkey,
    ADD CONSTRAINT sensor_data_sensor_id_fkey
    FOREIGN KEY (sensor_id) REFERENCES sensors(id) ON DELETE NO ACTION;
-- +goose StatementEnd
