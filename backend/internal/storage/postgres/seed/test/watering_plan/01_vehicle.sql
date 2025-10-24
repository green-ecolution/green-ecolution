-- +goose Up
-- +goose StatementBegin
INSERT INTO vehicles (id, number_plate, description, water_capacity, type, status, driving_license, model, width, height, length) VALUES (1, 'B-1234', 'Test vehicle 1', 100.0, 'trailer', 'active', 'BE', '1615/17 - Conrad - MAN TGE 3.180', 2.0, 1.5, 2.0);
INSERT INTO vehicles (id, number_plate, description, water_capacity, type, status, driving_license, model, width, height, length) VALUES (2, 'B-5678', 'Test vehicle 2', 150.0, 'transporter', 'unknown', 'C', 'Actros L Mercedes Benz', 2.4, 2.1, 5.0);
INSERT INTO vehicles (id, number_plate, description, water_capacity, type, status, driving_license, model, width, height, length) VALUES (3, 'A-1235', 'Test vehicle 3', 100.0, 'trailer', 'active', 'BE', '1615/18 - Conrad - MAN TGE 3.180', 2.0, 1.5, 2.0);
INSERT INTO vehicles (id, number_plate, description, water_capacity, type, status, driving_license, model, width, height, length) VALUES (4, 'A-5679', 'Test vehicle 4', 150.0, 'transporter', 'unknown', 'C', 'Actros M Mercedes Benz', 2.4, 2.1, 5.0);
ALTER SEQUENCE vehicles_id_seq RESTART WITH 3;


-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
DELETE FROM vehicles;
ALTER SEQUENCE vehicles_id_seq RESTART WITH 1;
-- +goose StatementEnd
