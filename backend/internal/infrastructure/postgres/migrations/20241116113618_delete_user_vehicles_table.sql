-- +goose Up
-- +goose StatementBegin
DROP TABLE IF EXISTS user_vehicles;
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
CREATE TABLE IF NOT EXISTS user_vehicles (
  user_id UUID NOT NULL,
  vehicle_id INT NOT NULL,
  PRIMARY KEY (user_id, vehicle_id),
  FOREIGN KEY (vehicle_id) REFERENCES vehicles(id)
);
-- +goose StatementEnd
