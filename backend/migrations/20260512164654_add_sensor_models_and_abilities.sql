CREATE TYPE sensor_ability_unit AS ENUM ('percent', 'centibar', 'ohm', 'celsius');
CREATE TYPE sensor_type         AS ENUM ('lorawan');

-- sensor_status: drop 'unknown', add 'prepared'. System is not yet in
-- production, so this is a clean swap (existing 'unknown' rows map to
-- 'prepared'; sensors must be re-activated via the new endpoint).
ALTER TABLE sensors ALTER COLUMN status DROP DEFAULT;
ALTER TABLE sensors ALTER COLUMN status TYPE TEXT;
DROP TYPE sensor_status;
CREATE TYPE sensor_status AS ENUM ('prepared', 'online', 'offline');
ALTER TABLE sensors
  ALTER COLUMN status TYPE sensor_status USING (
    CASE status WHEN 'unknown' THEN 'prepared' ELSE status END::sensor_status
  );
ALTER TABLE sensors ALTER COLUMN status SET DEFAULT 'prepared';

-- Drop coordinate columns: sensor position now flows from the linked tree.
ALTER TABLE sensors DROP COLUMN latitude;
ALTER TABLE sensors DROP COLUMN longitude;
ALTER TABLE sensors DROP COLUMN geometry;

CREATE TABLE sensor_abilities (
  id      SERIAL PRIMARY KEY,
  ability TEXT NOT NULL UNIQUE,
  unit    sensor_ability_unit NOT NULL
);

CREATE TABLE sensor_models (
  id          SERIAL PRIMARY KEY,
  name        TEXT NOT NULL UNIQUE,
  description TEXT,
  created_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TRIGGER update_sensor_models_updated_at
  BEFORE UPDATE ON sensor_models
  FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE sensor_model_abilities (
  id                SERIAL PRIMARY KEY,
  sensor_model_id   INT NOT NULL REFERENCES sensor_models(id)    ON DELETE CASCADE,
  sensor_ability_id INT NOT NULL REFERENCES sensor_abilities(id),
  depth_cm          INT NOT NULL,
  UNIQUE (sensor_model_id, sensor_ability_id, depth_cm)
);

CREATE TABLE sensor_lorawan (
  id            TEXT PRIMARY KEY REFERENCES sensors(id) ON DELETE CASCADE,
  serial_number TEXT NOT NULL,
  dev_eui       VARCHAR(16) NOT NULL,
  app_eui       VARCHAR(16) NOT NULL,
  app_key       VARCHAR(32) NOT NULL,
  at_pin        TEXT,
  ota_pin       TEXT,
  config        JSONB
);

CREATE TABLE sensor_data_ability_values (
  sensor_data_id          INT NOT NULL REFERENCES sensor_data(id)          ON DELETE CASCADE,
  sensor_model_ability_id INT NOT NULL REFERENCES sensor_model_abilities(id),
  value                   DECIMAL NOT NULL,
  PRIMARY KEY (sensor_data_id, sensor_model_ability_id)
);

ALTER TABLE sensors ADD COLUMN type     sensor_type;
ALTER TABLE sensors ADD COLUMN model_id INT REFERENCES sensor_models(id);

INSERT INTO sensor_abilities (ability, unit) VALUES
  ('soil_tension',  'centibar'),
  ('soil_moisture', 'percent'),
  ('temperature',   'celsius'),
  ('humidity',      'percent');

-- Explicit IDs so the backfill UPDATE below is deterministic.
INSERT INTO sensor_models (id, name, description) VALUES
  (1, 'EcoDrizzler',
      '3x Watermark (soil_tension @ 30/60/90 cm) + 1x SMT-100 (soil_moisture/temperature/humidity @ 15 cm). With on-board GPS.'),
  (2, 'GES-1000',
      'Dragino RS485-LB gateway with 2x capacitive soil-moisture probes (soil_moisture @ 30/90 cm). No GPS.');
SELECT setval('sensor_models_id_seq', (SELECT MAX(id) FROM sensor_models));

-- Name-based lookup so this is robust if ability ids shift.
INSERT INTO sensor_model_abilities (sensor_model_id, sensor_ability_id, depth_cm)
SELECT m.id, a.id, depth
FROM (VALUES
  ('EcoDrizzler', 'soil_tension',  30),
  ('EcoDrizzler', 'soil_tension',  60),
  ('EcoDrizzler', 'soil_tension',  90),
  ('EcoDrizzler', 'soil_moisture', 15),
  ('EcoDrizzler', 'temperature',   15),
  ('EcoDrizzler', 'humidity',      15),
  ('GES-1000',    'soil_moisture', 30),
  ('GES-1000',    'soil_moisture', 90)
) AS v(model_name, ability_name, depth)
JOIN sensor_models    m ON m.name    = v.model_name
JOIN sensor_abilities a ON a.ability = v.ability_name;

UPDATE sensors SET type = 'lorawan', model_id = 1;

ALTER TABLE sensors
  ALTER COLUMN type     SET NOT NULL,
  ALTER COLUMN model_id SET NOT NULL;

-- Placeholder rows for existing sensors; real credentials filled in later.
INSERT INTO sensor_lorawan (id, serial_number, dev_eui, app_eui, app_key)
SELECT id, '', '', '', '' FROM sensors;
