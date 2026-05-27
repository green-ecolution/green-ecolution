-- Correct probe depths (40/80 cm, not 30/90 cm) before the temperature
-- rows are inserted so the (model, ability, depth) UNIQUE index sees the
-- canonical values. battery uses depth_cm = 0 as a sentinel because the
-- column is NOT NULL and battery is a sensor-level metric.

INSERT INTO sensor_abilities (id, ability, unit) VALUES
  (uuidv7_from_timestamp(CURRENT_TIMESTAMP::timestamp), 'battery', 'volt')
ON CONFLICT (ability) DO NOTHING;

UPDATE sensor_model_abilities sma
SET depth_cm = CASE sma.depth_cm WHEN 30 THEN 40 WHEN 90 THEN 80 END
FROM sensor_models m, sensor_abilities a
WHERE sma.sensor_model_id   = m.id
  AND sma.sensor_ability_id = a.id
  AND m.name    = 'GES-1000'
  AND a.ability = 'soil_moisture'
  AND sma.depth_cm IN (30, 90);

INSERT INTO sensor_model_abilities (id, sensor_model_id, sensor_ability_id, depth_cm)
SELECT uuidv7_from_timestamp(CURRENT_TIMESTAMP::timestamp), m.id, a.id, v.depth
FROM (VALUES
  ('GES-1000', 'temperature', 40),
  ('GES-1000', 'temperature', 80),
  ('GES-1000', 'battery',      0)
) AS v(model_name, ability_name, depth)
JOIN sensor_models    m ON m.name    = v.model_name
JOIN sensor_abilities a ON a.ability = v.ability_name
ON CONFLICT (sensor_model_id, sensor_ability_id, depth_cm) DO NOTHING;

UPDATE sensor_models
SET description = 'Dragino RS485-LB gateway with 2x capacitive probes (soil_moisture/temperature @ 40/80 cm) + battery voltage. No GPS.'
WHERE name = 'GES-1000';
