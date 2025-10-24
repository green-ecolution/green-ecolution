-- +goose Up
-- +goose StatementBegin
INSERT INTO vehicles (id, number_plate, description, water_capacity, type, status, driving_license, model, width, height, length, weight) VALUES
  (1, 'B-1234', 'Test vehicle 1', 100.0, 'trailer', 'active', 'BE', '1615/17 - Conrad - MAN TGE 3.180', 2.0, 1.5, 2.0, 3.3),
  (2, 'B-5678', 'Test vehicle 2', 150.0, 'transporter', 'unknown', 'C', 'Actros L Mercedes Benz', 2.4, 2.1, 5.0, 3.7);

INSERT INTO vehicles (id, number_plate, description, water_capacity, type, status, driving_license, model, width, height, length, weight, provider, additional_informations) VALUES
  (3, 'B-1001', 'Test vehicle 3', 150.0, 'transporter', 'unknown', 'C', 'Actros L Mercedes Benz', 2.4, 2.1, 5.0, 3.7, 'test-provider', '{"foo":"bar"}');

ALTER SEQUENCE vehicles_id_seq RESTART WITH 4;


-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
DELETE FROM vehicles;
ALTER SEQUENCE vehicles_id_seq RESTART WITH 1;
-- +goose StatementEnd
