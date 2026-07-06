-- A sensor must never be linked to two trees at once: MQTT ingest resolves
-- readings via `trees.sensor_id`, so a duplicate link routes sensor data to an
-- arbitrary tree. The service layer now rejects duplicate attachments; this
-- unique index is the database-level backstop.

-- Defensive dedup: should duplicates exist, the newest tree (uuid v7 ids are
-- time-ordered) keeps the sensor, older links are cleared.
UPDATE trees t
   SET sensor_id = NULL
 WHERE t.sensor_id IS NOT NULL
   AND EXISTS (
        SELECT 1
          FROM trees newer
         WHERE newer.sensor_id = t.sensor_id
           AND newer.id > t.id
   );

DROP INDEX IF EXISTS idx_trees_sensor_id;
CREATE UNIQUE INDEX idx_trees_sensor_id
    ON trees (sensor_id) WHERE sensor_id IS NOT NULL;
