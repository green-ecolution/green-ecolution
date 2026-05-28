-- Add 'volt' to `sensor_ability_unit` so the next migration can register a
-- `battery` ability. Kept in its own file because Postgres requires that
-- new enum values be committed before they can be referenced.
ALTER TYPE sensor_ability_unit ADD VALUE IF NOT EXISTS 'volt';
