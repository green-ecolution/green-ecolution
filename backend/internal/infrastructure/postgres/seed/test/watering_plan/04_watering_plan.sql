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

INSERT INTO tree_cluster_watering_plans (tree_cluster_id, watering_plan_id, consumed_water) VALUES 
(1, 1, 10.0),
(2, 1, 10.0),
(3, 2, 10.0),
(1, 3, 10.0),
(2, 3, 10.0),
(3, 3, 10.0),
(3, 4, 10.0),
(3, 5, 10.0);

INSERT INTO user_watering_plans (user_id, watering_plan_id) VALUES 
('6a1078e8-80fd-458f-b74e-e388fe2dd6ab', 1),
('05c028d9-62ef-4dcc-aa79-6b2fe9ce6f42', 1),
('e5ed176c-3aa8-4676-8e5b-0a0001a1bb88', 1),
('6a1078e8-80fd-458f-b74e-e388fe2dd6ab', 2),
('6a1078e8-80fd-458f-b74e-e388fe2dd6ab', 3),
('6a1078e8-80fd-458f-b74e-e388fe2dd6ab', 4),
('6a1078e8-80fd-458f-b74e-e388fe2dd6ab', 5);
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
DELETE FROM watering_plans;
DELETE FROM vehicle_watering_plans;
DELETE FROM tree_cluster_watering_plans;
-- +goose StatementEnd
