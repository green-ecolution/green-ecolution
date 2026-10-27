INSERT INTO sensor_models (id, name, description)
VALUES (uuidv7_from_timestamp(CURRENT_TIMESTAMP::timestamp), 'Test-Level', 'Distance sensor of the test module');

INSERT INTO sensor_model_abilities (id, sensor_model_id, sensor_ability_id, depth_cm)
SELECT uuidv7_from_timestamp(CURRENT_TIMESTAMP::timestamp), m.id, a.id, 0
FROM sensor_models m
JOIN sensor_abilities a ON a.ability = 'distance'
WHERE m.name = 'Test-Level';
