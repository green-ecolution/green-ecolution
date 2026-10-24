CREATE TABLE sensor_attachments (
    sensor_id   VARCHAR PRIMARY KEY REFERENCES sensors (id) ON DELETE CASCADE,
    module_key  TEXT NOT NULL CHECK (module_key ~ '^[a-z][a-z0-9_]{0,31}$'),
    target_kind TEXT NOT NULL CHECK (target_kind ~ '^[a-z][a-z0-9_]{0,31}$'),
    target_id   UUID NOT NULL,
    -- One sensor per object, across modules.
    CONSTRAINT sensor_attachments_target_unique UNIQUE (module_key, target_kind, target_id)
);

INSERT INTO sensor_attachments (sensor_id, module_key, target_kind, target_id)
SELECT sensor_id, 'irrigation', 'tree', id
FROM trees
WHERE sensor_id IS NOT NULL AND sensor_id <> '';

DROP INDEX IF EXISTS idx_trees_sensor_id;
ALTER TABLE trees DROP CONSTRAINT IF EXISTS trees_sensor_id_fkey;
ALTER TABLE trees DROP COLUMN sensor_id;
