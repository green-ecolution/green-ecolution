-- Core measurement vocabulary; the sensor models that use it come with their modules.
INSERT INTO sensor_abilities (id, ability, unit) VALUES
  (uuidv7_from_timestamp(CURRENT_TIMESTAMP::timestamp), 'distance', 'centimeter')
ON CONFLICT (ability) DO NOTHING;
