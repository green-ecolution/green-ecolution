-- +goose Up
-- +goose StatementBegin
INSERT INTO watering_plans (id, date, description, status, distance, total_water_required, cancellation_note)
VALUES 
  (1, '2024-09-22', 'New watering plan for the west side of the city', 'planned', 63.0, 720, ''),
  (3, '2024-06-12', 'Very important watering plan due to no rainfall', 'finished', 63.0, 0, ''),
  (4, '2024-06-10', 'New watering plan for the south side of the city', 'not competed', 63.0, 0, ''),
  (5, '2024-06-04', 'Canceled due to flood', 'canceled', 63.0, 0.0, 'The watering plan was cancelled due to various reasons.');

INSERT INTO watering_plans (id, date, description, status, distance, total_water_required, cancellation_note, provider, additional_informations)
VALUES 
  (2, '2024-08-03', 'New watering plan for the east side of the city', 'active', 63.0, 0, '', 'test-provider', '{"foo":"bar"}');

ALTER SEQUENCE watering_plans_id_seq RESTART WITH 7;

INSERT INTO vehicle_watering_plans (vehicle_id, watering_plan_id) VALUES 
(1, 1),
(2, 1),
(2, 2),
(2, 3),
(2, 4),
(2, 5);
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
DELETE FROM vehicle_watering_plans;
DELETE FROM watering_plans;
ALTER SEQUENCE watering_plans_id_seq RESTART WITH 1;
-- +goose StatementEnd