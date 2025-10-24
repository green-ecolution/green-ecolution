-- +goose Up
ALTER TABLE sensor_data DROP CONSTRAINT sensor_data_sensor_id_fkey;
ALTER TABLE trees DROP CONSTRAINT trees_sensor_id_fkey;

ALTER TABLE sensors DROP CONSTRAINT sensors_pkey;
ALTER TABLE sensors ALTER COLUMN id TYPE VARCHAR USING id::TEXT;
ALTER TABLE sensors ADD PRIMARY KEY (id);
ALTER TABLE sensors ADD COLUMN latitude FLOAT NOT NULL;
ALTER TABLE sensors ADD COLUMN longitude FLOAT NOT NULL;
ALTER TABLE sensors ADD COLUMN geometry GEOMETRY(Point, 4326);

ALTER TABLE sensor_data ALTER COLUMN sensor_id TYPE VARCHAR USING sensor_id::TEXT;
ALTER TABLE sensor_data ADD CONSTRAINT sensor_data_sensor_id_fkey FOREIGN KEY (sensor_id) REFERENCES sensors(id) ON DELETE CASCADE;

ALTER TABLE trees ALTER COLUMN sensor_id TYPE VARCHAR USING sensor_id::TEXT;
ALTER TABLE trees ADD CONSTRAINT trees_sensor_id_fkey FOREIGN KEY (sensor_id) REFERENCES sensors(id);

ALTER TABLE flowerbeds ALTER COLUMN sensor_id TYPE VARCHAR USING sensor_id::TEXT;