INSERT INTO sensor_models (id, name, description)
VALUES (gen_random_uuid(), 'GE Distance (simuliert)',
        'Simulated ultrasonic level sensor: distance from the lid to the fill surface in cm, plus battery voltage.')
ON CONFLICT (name) DO NOTHING;

INSERT INTO sensor_model_abilities (id, sensor_model_id, sensor_ability_id, depth_cm)
SELECT gen_random_uuid(), m.id, a.id, 0
FROM sensor_models m
JOIN sensor_abilities a ON a.ability IN ('distance', 'battery')
WHERE m.name = 'GE Distance (simuliert)'
ON CONFLICT (sensor_model_id, sensor_ability_id, depth_cm) DO NOTHING;
