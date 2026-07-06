-- Migrate every SERIAL/INT primary key (and the FKs that reference them) to
-- UUID v7. Each row's UUID is derived from its `created_at` so the embedded
-- v7 timestamp prefix preserves chronological ordering; the column is then
-- dropped from each rebuilt table because the value is recoverable via
-- `uuidv7_timestamp(uuid)` below or by extracting the leading 48 bits
-- client-side.
--
-- This migration also rewrites every affected table in place so the new
-- columns sit in a clean canonical order (id, updated_at, then payload).
-- PostgreSQL has no ALTER COLUMN ... SET POSITION, so the only way to
-- achieve that is `CREATE TABLE x_new`, copy, `DROP TABLE x CASCADE`,
-- rename back. We exploit that single rewrite to do the UUID conversion
-- and the column reorder in one shot — no double rebuild.
--
-- Out of scope: sensors.id (LoRaWAN EUI, already TEXT) — the *column* type
-- stays text, but the table is still rebuilt so model_id sits in the right
-- place. User UUIDs are owned by Keycloak.
--
-- One-way migration. Take a `pg_dump -F c` before running `just migrate-up`.

BEGIN;

-- ---------------------------------------------------------------------------
-- 1) Helper functions
-- ---------------------------------------------------------------------------

CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Build a UUID v7 whose 48-bit timestamp prefix encodes the given moment.
-- The remaining 74 bits are random. Layout per RFC 9562:
--   bytes 0-5 : unix_ts_ms (big-endian, 48 bits)
--   byte  6   : 0x70 | rand_a_hi (version 7, 4 bits + 4 bits rand)
--   byte  7   : rand_a_lo (8 bits rand)
--   byte  8   : 0x80 | rand_b_hi6 (variant 10, 2 bits + 6 bits rand)
--   bytes 9-15: rand_b_lo (56 bits rand)
--
-- Assumption: `created_at` columns are TIMESTAMP WITHOUT TIME ZONE and
-- Postgres' `extract(epoch from ts)` treats them as UTC — matches our
-- CURRENT_TIMESTAMP default in a UTC-configured container.
CREATE OR REPLACE FUNCTION uuidv7_from_timestamp(ts timestamp)
RETURNS uuid
LANGUAGE plpgsql
AS $$
DECLARE
    unix_ts_ms bigint;
    bytes      bytea;
BEGIN
    unix_ts_ms := (extract(epoch from ts) * 1000)::bigint;
    bytes      := gen_random_bytes(16);

    bytes := set_byte(bytes, 0, ((unix_ts_ms >> 40) & 255)::int);
    bytes := set_byte(bytes, 1, ((unix_ts_ms >> 32) & 255)::int);
    bytes := set_byte(bytes, 2, ((unix_ts_ms >> 24) & 255)::int);
    bytes := set_byte(bytes, 3, ((unix_ts_ms >> 16) & 255)::int);
    bytes := set_byte(bytes, 4, ((unix_ts_ms >>  8) & 255)::int);
    bytes := set_byte(bytes, 5, ( unix_ts_ms        & 255)::int);

    -- Version 7 in the high nibble of byte 6.
    bytes := set_byte(bytes, 6, (get_byte(bytes, 6) & 15)  | 112);
    -- Variant 10 in the top two bits of byte 8.
    bytes := set_byte(bytes, 8, (get_byte(bytes, 8) & 63)  | 128);

    RETURN encode(bytes, 'hex')::uuid;
END;
$$;

-- Inverse: pull the original instant back out of a v7 id. Returns NULL for
-- non-v7 UUIDs. Used by application code that still needs `created_at`.
CREATE OR REPLACE FUNCTION uuidv7_timestamp(u uuid)
RETURNS timestamp
LANGUAGE plpgsql
IMMUTABLE
AS $$
DECLARE
    bytes      bytea;
    unix_ts_ms bigint;
BEGIN
    bytes := decode(replace(u::text, '-', ''), 'hex');
    IF (get_byte(bytes, 6) >> 4) <> 7 THEN
        RETURN NULL;
    END IF;

    unix_ts_ms :=
        (get_byte(bytes, 0)::bigint << 40) |
        (get_byte(bytes, 1)::bigint << 32) |
        (get_byte(bytes, 2)::bigint << 24) |
        (get_byte(bytes, 3)::bigint << 16) |
        (get_byte(bytes, 4)::bigint <<  8) |
         get_byte(bytes, 5)::bigint;

    RETURN to_timestamp(unix_ts_ms / 1000.0) AT TIME ZONE 'UTC';
END;
$$;

-- ---------------------------------------------------------------------------
-- 2) Materialise int -> uuid mappings for every parent table. We need
--    these in temp tables before any DROP/RENAME so the JOINs in the
--    INSERT INTO ... SELECT phase have somewhere to look the new uuid up.
-- ---------------------------------------------------------------------------

CREATE TEMP TABLE map_regions ON COMMIT DROP AS
    SELECT id AS old_int_id, uuidv7_from_timestamp(created_at) AS new_uuid FROM regions;
CREATE INDEX ON map_regions (old_int_id);

CREATE TEMP TABLE map_tree_clusters ON COMMIT DROP AS
    SELECT id AS old_int_id, uuidv7_from_timestamp(created_at) AS new_uuid FROM tree_clusters;
CREATE INDEX ON map_tree_clusters (old_int_id);

CREATE TEMP TABLE map_trees ON COMMIT DROP AS
    SELECT id AS old_int_id, uuidv7_from_timestamp(created_at) AS new_uuid FROM trees;
CREATE INDEX ON map_trees (old_int_id);

CREATE TEMP TABLE map_vehicles ON COMMIT DROP AS
    SELECT id AS old_int_id, uuidv7_from_timestamp(created_at) AS new_uuid FROM vehicles;
CREATE INDEX ON map_vehicles (old_int_id);

CREATE TEMP TABLE map_watering_plans ON COMMIT DROP AS
    SELECT id AS old_int_id, uuidv7_from_timestamp(created_at) AS new_uuid FROM watering_plans;
CREATE INDEX ON map_watering_plans (old_int_id);

CREATE TEMP TABLE map_sensor_models ON COMMIT DROP AS
    SELECT id AS old_int_id, uuidv7_from_timestamp(created_at) AS new_uuid FROM sensor_models;
CREATE INDEX ON map_sensor_models (old_int_id);

CREATE TEMP TABLE map_sensor_data ON COMMIT DROP AS
    SELECT id AS old_int_id, uuidv7_from_timestamp(created_at) AS new_uuid FROM sensor_data;
CREATE INDEX ON map_sensor_data (old_int_id);

-- No `created_at` on these two — pick clock_timestamp() at migration time.
-- The relative ordering is preserved (rows are inserted microseconds apart).
CREATE TEMP TABLE map_sensor_abilities ON COMMIT DROP AS
    SELECT id AS old_int_id,
           uuidv7_from_timestamp(clock_timestamp()::timestamp) AS new_uuid
      FROM sensor_abilities;
CREATE INDEX ON map_sensor_abilities (old_int_id);

CREATE TEMP TABLE map_sensor_model_abilities ON COMMIT DROP AS
    SELECT id AS old_int_id,
           uuidv7_from_timestamp(clock_timestamp()::timestamp) AS new_uuid
      FROM sensor_model_abilities;
CREATE INDEX ON map_sensor_model_abilities (old_int_id);

-- ---------------------------------------------------------------------------
-- 3) Drop every FK that crosses table boundaries. With no FKs in flight,
--    each rebuild step can DROP TABLE x and INSERT INTO x_new freely.
-- ---------------------------------------------------------------------------

ALTER TABLE trees                       DROP CONSTRAINT IF EXISTS trees_tree_cluster_id_fkey;
ALTER TABLE trees                       DROP CONSTRAINT IF EXISTS trees_sensor_id_fkey;
ALTER TABLE tree_clusters               DROP CONSTRAINT IF EXISTS tree_clusters_region_id_fkey;
ALTER TABLE sensors                     DROP CONSTRAINT IF EXISTS sensors_model_id_fkey;
ALTER TABLE sensor_data                 DROP CONSTRAINT IF EXISTS sensor_data_sensor_id_fkey;
ALTER TABLE sensor_model_abilities      DROP CONSTRAINT IF EXISTS sensor_model_abilities_sensor_model_id_fkey;
ALTER TABLE sensor_model_abilities      DROP CONSTRAINT IF EXISTS sensor_model_abilities_sensor_ability_id_fkey;
ALTER TABLE sensor_data_ability_values  DROP CONSTRAINT IF EXISTS sensor_data_ability_values_sensor_data_id_fkey;
ALTER TABLE sensor_data_ability_values  DROP CONSTRAINT IF EXISTS sensor_data_ability_values_sensor_model_ability_id_fkey;
ALTER TABLE user_watering_plans         DROP CONSTRAINT IF EXISTS user_watering_plans_watering_plan_id_fkey;
ALTER TABLE vehicle_watering_plans      DROP CONSTRAINT IF EXISTS vehicle_watering_plans_vehicle_id_fkey;
ALTER TABLE vehicle_watering_plans      DROP CONSTRAINT IF EXISTS vehicle_watering_plans_watering_plan_id_fkey;
ALTER TABLE tree_cluster_watering_plans DROP CONSTRAINT IF EXISTS tree_cluster_watering_plans_tree_cluster_id_fkey;
ALTER TABLE tree_cluster_watering_plans DROP CONSTRAINT IF EXISTS tree_cluster_watering_plans_watering_plan_id_fkey;
ALTER TABLE sensor_lorawan              DROP CONSTRAINT IF EXISTS sensor_lorawan_id_fkey;

-- The composite-PK on sensor_data_ability_values references columns we're
-- about to drop, so the PK has to go too. It's recreated in step 4.
ALTER TABLE sensor_data_ability_values  DROP CONSTRAINT IF EXISTS sensor_data_ability_values_pkey;

-- ---------------------------------------------------------------------------
-- 4) Rebuild every affected table with the canonical column order:
--    id first, updated_at second, payload columns after. The `_new` table
--    is populated from the old one through the mapping temp tables; then we
--    drop the original and rename. Indexes and triggers are recreated
--    explicitly. FKs are left for step 5 so we don't have to worry about
--    declaration order here.
-- ---------------------------------------------------------------------------

-- == regions ==
CREATE TABLE regions_new (
    id         uuid                       NOT NULL PRIMARY KEY,
    updated_at timestamp                  NOT NULL DEFAULT CURRENT_TIMESTAMP,
    name       varchar(255)               NOT NULL,
    geometry   geometry(MultiPolygon,4326)
);
INSERT INTO regions_new (id, updated_at, name, geometry)
    SELECT m.new_uuid, r.updated_at, r.name, r.geometry
      FROM regions r
      JOIN map_regions m ON m.old_int_id = r.id;
DROP TABLE regions CASCADE;
ALTER TABLE regions_new RENAME TO regions;
ALTER INDEX regions_new_pkey RENAME TO regions_pkey;
CREATE INDEX idx_regions_geometry ON regions USING GIST (geometry);
CREATE TRIGGER update_region_updated_at
    BEFORE UPDATE ON regions FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- == tree_clusters ==
CREATE TABLE tree_clusters_new (
    id                      uuid                  NOT NULL PRIMARY KEY,
    updated_at              timestamp             NOT NULL DEFAULT CURRENT_TIMESTAMP,
    region_id               uuid,
    name                    text                  NOT NULL,
    address                 text                  NOT NULL,
    description             text                  NOT NULL,
    moisture_level          double precision      NOT NULL,
    soil_condition          tree_soil_condition   NOT NULL DEFAULT 'unknown',
    watering_status         watering_status       NOT NULL DEFAULT 'unknown',
    last_watered            timestamp,
    archived                boolean               NOT NULL DEFAULT false,
    latitude                double precision,
    longitude               double precision,
    geometry                geometry(Point,4326),
    provider                text,
    additional_informations jsonb
);
INSERT INTO tree_clusters_new (
    id, updated_at, region_id, name, address, description, moisture_level,
    soil_condition, watering_status, last_watered, archived,
    latitude, longitude, geometry, provider, additional_informations
)
    SELECT m.new_uuid, tc.updated_at,
           mr.new_uuid,
           tc.name, tc.address, tc.description, tc.moisture_level,
           tc.soil_condition, tc.watering_status, tc.last_watered, tc.archived,
           tc.latitude, tc.longitude, tc.geometry, tc.provider, tc.additional_informations
      FROM tree_clusters tc
      JOIN map_tree_clusters m ON m.old_int_id = tc.id
      LEFT JOIN map_regions   mr ON mr.old_int_id = tc.region_id;
DROP TABLE tree_clusters CASCADE;
ALTER TABLE tree_clusters_new RENAME TO tree_clusters;
ALTER INDEX tree_clusters_new_pkey RENAME TO tree_clusters_pkey;
CREATE INDEX idx_tree_clusters_region_id
    ON tree_clusters (region_id) WHERE region_id IS NOT NULL;
CREATE INDEX idx_tree_clusters_provider        ON tree_clusters (provider);
CREATE INDEX idx_tree_clusters_watering_status ON tree_clusters (watering_status);
CREATE TRIGGER update_tree_clusters_updated_at
    BEFORE UPDATE ON tree_clusters FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- == sensor_models ==
CREATE TABLE sensor_models_new (
    id          uuid       NOT NULL PRIMARY KEY,
    updated_at  timestamp  NOT NULL DEFAULT CURRENT_TIMESTAMP,
    name        text       NOT NULL UNIQUE,
    description text
);
INSERT INTO sensor_models_new (id, updated_at, name, description)
    SELECT m.new_uuid, sm.updated_at, sm.name, sm.description
      FROM sensor_models sm
      JOIN map_sensor_models m ON m.old_int_id = sm.id;
DROP TABLE sensor_models CASCADE;
ALTER TABLE sensor_models_new RENAME TO sensor_models;
ALTER INDEX sensor_models_new_pkey     RENAME TO sensor_models_pkey;
ALTER INDEX sensor_models_new_name_key RENAME TO sensor_models_name_key;
CREATE TRIGGER update_sensor_models_updated_at
    BEFORE UPDATE ON sensor_models FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- == sensor_abilities ==
CREATE TABLE sensor_abilities_new (
    id      uuid                NOT NULL PRIMARY KEY,
    ability text                NOT NULL UNIQUE,
    unit    sensor_ability_unit NOT NULL
);
INSERT INTO sensor_abilities_new (id, ability, unit)
    SELECT m.new_uuid, sa.ability, sa.unit
      FROM sensor_abilities sa
      JOIN map_sensor_abilities m ON m.old_int_id = sa.id;
DROP TABLE sensor_abilities CASCADE;
ALTER TABLE sensor_abilities_new RENAME TO sensor_abilities;
ALTER INDEX sensor_abilities_new_pkey        RENAME TO sensor_abilities_pkey;
ALTER INDEX sensor_abilities_new_ability_key RENAME TO sensor_abilities_ability_key;

-- == sensor_model_abilities ==
CREATE TABLE sensor_model_abilities_new (
    id                uuid    NOT NULL PRIMARY KEY,
    sensor_model_id   uuid    NOT NULL,
    sensor_ability_id uuid    NOT NULL,
    depth_cm          integer NOT NULL,
    CONSTRAINT sensor_model_abilities_new_unique
        UNIQUE (sensor_model_id, sensor_ability_id, depth_cm)
);
INSERT INTO sensor_model_abilities_new (id, sensor_model_id, sensor_ability_id, depth_cm)
    SELECT m.new_uuid, mm.new_uuid, ma.new_uuid, sma.depth_cm
      FROM sensor_model_abilities sma
      JOIN map_sensor_model_abilities m  ON m.old_int_id  = sma.id
      JOIN map_sensor_models          mm ON mm.old_int_id = sma.sensor_model_id
      JOIN map_sensor_abilities       ma ON ma.old_int_id = sma.sensor_ability_id;
DROP TABLE sensor_model_abilities CASCADE;
ALTER TABLE sensor_model_abilities_new RENAME TO sensor_model_abilities;
ALTER INDEX sensor_model_abilities_new_pkey   RENAME TO sensor_model_abilities_pkey;
ALTER INDEX sensor_model_abilities_new_unique RENAME TO sensor_model_abilities_unique;

-- == sensors (id stays TEXT — out of scope for the UUID swap, but the table
--    is rebuilt so model_id sits in canonical position. `created_at` stays
--    here because the id is a LoRaWAN EUI, not a v7 UUID, so the timestamp
--    isn't recoverable from it.) ==
CREATE TABLE sensors_new (
    id                      character varying NOT NULL PRIMARY KEY,
    created_at              timestamp         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at              timestamp         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    model_id                uuid              NOT NULL,
    type                    sensor_type       NOT NULL,
    status                  sensor_status     NOT NULL DEFAULT 'prepared',
    provider                text,
    additional_informations jsonb
);
INSERT INTO sensors_new (
    id, created_at, updated_at, model_id, type, status, provider, additional_informations
)
    SELECT s.id, s.created_at, s.updated_at,
           mm.new_uuid, s.type, s.status, s.provider, s.additional_informations
      FROM sensors s
      JOIN map_sensor_models mm ON mm.old_int_id = s.model_id;
DROP TABLE sensors CASCADE;
ALTER TABLE sensors_new RENAME TO sensors;
ALTER INDEX sensors_new_pkey RENAME TO sensors_pkey;
CREATE INDEX idx_sensors_provider ON sensors (provider);
CREATE TRIGGER update_sensors_updated_at
    BEFORE UPDATE ON sensors FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- == sensor_lorawan (id stays text, table not rebuilt) ==
--    `DROP TABLE sensors CASCADE` above only drops *dependent objects*
--    (FK constraints, views) — it does not drop `sensor_lorawan` itself,
--    so its rows survive. The FK to `sensors.id` is recreated in step 5.

-- == sensor_data ==
CREATE TABLE sensor_data_new (
    id         uuid              NOT NULL PRIMARY KEY,
    updated_at timestamp         NOT NULL DEFAULT CURRENT_TIMESTAMP,
    sensor_id  character varying NOT NULL,
    data       jsonb             NOT NULL
);
INSERT INTO sensor_data_new (id, updated_at, sensor_id, data)
    SELECT m.new_uuid, sd.updated_at, sd.sensor_id, sd.data
      FROM sensor_data sd
      JOIN map_sensor_data m ON m.old_int_id = sd.id;
DROP TABLE sensor_data CASCADE;
ALTER TABLE sensor_data_new RENAME TO sensor_data;
ALTER INDEX sensor_data_new_pkey RENAME TO sensor_data_pkey;
CREATE INDEX idx_sensor_data_sensor_id_desc ON sensor_data (sensor_id, id DESC);
CREATE TRIGGER update_sensor_data_updated_at
    BEFORE UPDATE ON sensor_data FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- == sensor_data_ability_values (junction; rebuilt for canonical FK-first
--    column order) ==
CREATE TABLE sensor_data_ability_values_new (
    sensor_data_id          uuid    NOT NULL,
    sensor_model_ability_id uuid    NOT NULL,
    value                   numeric NOT NULL,
    PRIMARY KEY (sensor_data_id, sensor_model_ability_id)
);
INSERT INTO sensor_data_ability_values_new (sensor_data_id, sensor_model_ability_id, value)
    SELECT md.new_uuid, mma.new_uuid, sdav.value
      FROM sensor_data_ability_values sdav
      JOIN map_sensor_data           md  ON md.old_int_id  = sdav.sensor_data_id
      JOIN map_sensor_model_abilities mma ON mma.old_int_id = sdav.sensor_model_ability_id;
DROP TABLE sensor_data_ability_values;
ALTER TABLE sensor_data_ability_values_new RENAME TO sensor_data_ability_values;
ALTER INDEX sensor_data_ability_values_new_pkey
    RENAME TO sensor_data_ability_values_pkey;

-- == vehicles ==
CREATE TABLE vehicles_new (
    id                      uuid             NOT NULL PRIMARY KEY,
    updated_at              timestamp        NOT NULL DEFAULT CURRENT_TIMESTAMP,
    archived_at             timestamp,
    number_plate            text             NOT NULL CONSTRAINT vehicles_new_unique_number_plate UNIQUE,
    model                   text             NOT NULL,
    description             text             NOT NULL,
    type                    vehicle_type     NOT NULL DEFAULT 'transporter',
    status                  vehicle_status   NOT NULL DEFAULT 'unknown',
    driving_license         driving_license  NOT NULL DEFAULT 'B',
    water_capacity          double precision NOT NULL,
    width                   double precision NOT NULL DEFAULT 0,
    height                  double precision NOT NULL DEFAULT 0,
    length                  double precision NOT NULL DEFAULT 0,
    weight                  double precision NOT NULL DEFAULT 0,
    provider                text,
    additional_informations jsonb
);
INSERT INTO vehicles_new (
    id, updated_at, archived_at, number_plate, model, description,
    type, status, driving_license,
    water_capacity, width, height, length, weight,
    provider, additional_informations
)
    SELECT m.new_uuid, v.updated_at, v.archived_at, v.number_plate, v.model, v.description,
           v.type, v.status, v.driving_license,
           v.water_capacity, v.width, v.height, v.length, v.weight,
           v.provider, v.additional_informations
      FROM vehicles v
      JOIN map_vehicles m ON m.old_int_id = v.id;
DROP TABLE vehicles CASCADE;
ALTER TABLE vehicles_new RENAME TO vehicles;
ALTER INDEX vehicles_new_pkey                  RENAME TO vehicles_pkey;
ALTER INDEX vehicles_new_unique_number_plate   RENAME TO unique_number_plate;
CREATE INDEX idx_vehicles_provider    ON vehicles (provider);
CREATE INDEX idx_vehicles_type        ON vehicles (type);
CREATE INDEX idx_vehicles_archived_at ON vehicles (archived_at);
CREATE TRIGGER update_vehicles_updated_at
    BEFORE UPDATE ON vehicles FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- == watering_plans ==
CREATE TABLE watering_plans_new (
    id                      uuid                 NOT NULL PRIMARY KEY,
    updated_at              timestamp            NOT NULL DEFAULT CURRENT_TIMESTAMP,
    date                    date                 NOT NULL,
    status                  watering_plan_status NOT NULL DEFAULT 'unknown',
    description             text                 NOT NULL,
    distance                double precision,
    total_water_required    double precision,
    duration                double precision     NOT NULL DEFAULT 0,
    refill_count            integer              NOT NULL DEFAULT 0,
    gpx_url                 text,
    cancellation_note       text                 NOT NULL DEFAULT '',
    provider                text,
    additional_informations jsonb
);
INSERT INTO watering_plans_new (
    id, updated_at, date, status, description, distance, total_water_required,
    duration, refill_count, gpx_url, cancellation_note,
    provider, additional_informations
)
    SELECT m.new_uuid, wp.updated_at, wp.date, wp.status, wp.description,
           wp.distance, wp.total_water_required,
           wp.duration, wp.refill_count, wp.gpx_url, wp.cancellation_note,
           wp.provider, wp.additional_informations
      FROM watering_plans wp
      JOIN map_watering_plans m ON m.old_int_id = wp.id;
DROP TABLE watering_plans CASCADE;
ALTER TABLE watering_plans_new RENAME TO watering_plans;
ALTER INDEX watering_plans_new_pkey RENAME TO watering_plans_pkey;
CREATE INDEX idx_watering_plans_provider ON watering_plans (provider);
CREATE INDEX idx_watering_plans_date     ON watering_plans (date DESC);
CREATE TRIGGER update_watering_plans_updated_at
    BEFORE UPDATE ON watering_plans FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- == trees ==
CREATE TABLE trees_new (
    id                      uuid                  NOT NULL PRIMARY KEY,
    updated_at              timestamp             NOT NULL DEFAULT CURRENT_TIMESTAMP,
    tree_cluster_id         uuid,
    sensor_id               character varying,
    number                  text                  NOT NULL,
    species                 text                  NOT NULL,
    planting_year           integer               NOT NULL,
    latitude                double precision      NOT NULL,
    longitude               double precision      NOT NULL,
    geometry                geometry(Point,4326),
    watering_status         watering_status       NOT NULL DEFAULT 'unknown',
    last_watered            timestamp,
    description             text,
    provider                text,
    additional_informations jsonb
);
INSERT INTO trees_new (
    id, updated_at, tree_cluster_id, sensor_id,
    number, species, planting_year, latitude, longitude, geometry,
    watering_status, last_watered, description, provider, additional_informations
)
    SELECT m.new_uuid, t.updated_at,
           mc.new_uuid, t.sensor_id,
           t.number, t.species, t.planting_year, t.latitude, t.longitude, t.geometry,
           t.watering_status, t.last_watered, t.description, t.provider, t.additional_informations
      FROM trees t
      JOIN map_trees           m  ON m.old_int_id  = t.id
      LEFT JOIN map_tree_clusters mc ON mc.old_int_id = t.tree_cluster_id;
DROP TABLE trees CASCADE;
ALTER TABLE trees_new RENAME TO trees;
ALTER INDEX trees_new_pkey RENAME TO trees_pkey;
CREATE INDEX idx_trees_cluster_id
    ON trees (tree_cluster_id) WHERE tree_cluster_id IS NOT NULL;
CREATE INDEX idx_trees_sensor_id
    ON trees (sensor_id) WHERE sensor_id IS NOT NULL;
CREATE INDEX idx_trees_provider        ON trees (provider);
CREATE INDEX idx_trees_watering_status ON trees (watering_status);
CREATE INDEX idx_trees_geometry        ON trees USING GIST (geometry);
CREATE TRIGGER update_trees_updated_at
    BEFORE UPDATE ON trees FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- == tree_cluster_watering_plans (junction; rebuild puts FK columns first) ==
CREATE TABLE tree_cluster_watering_plans_new (
    tree_cluster_id  uuid             NOT NULL,
    watering_plan_id uuid             NOT NULL,
    consumed_water   double precision NOT NULL DEFAULT 0.0,
    PRIMARY KEY (tree_cluster_id, watering_plan_id)
);
INSERT INTO tree_cluster_watering_plans_new (tree_cluster_id, watering_plan_id, consumed_water)
    SELECT mc.new_uuid, mp.new_uuid, tcwp.consumed_water
      FROM tree_cluster_watering_plans tcwp
      JOIN map_tree_clusters   mc ON mc.old_int_id = tcwp.tree_cluster_id
      JOIN map_watering_plans  mp ON mp.old_int_id = tcwp.watering_plan_id;
DROP TABLE tree_cluster_watering_plans;
ALTER TABLE tree_cluster_watering_plans_new RENAME TO tree_cluster_watering_plans;
ALTER INDEX tree_cluster_watering_plans_new_pkey
    RENAME TO tree_cluster_watering_plans_pkey;

-- == vehicle_watering_plans (junction; columns already in canonical order
--    but we rewrite to convert ints to uuids consistently) ==
CREATE TABLE vehicle_watering_plans_new (
    vehicle_id       uuid NOT NULL,
    watering_plan_id uuid NOT NULL,
    PRIMARY KEY (vehicle_id, watering_plan_id)
);
INSERT INTO vehicle_watering_plans_new (vehicle_id, watering_plan_id)
    SELECT mv.new_uuid, mp.new_uuid
      FROM vehicle_watering_plans vwp
      JOIN map_vehicles       mv ON mv.old_int_id = vwp.vehicle_id
      JOIN map_watering_plans mp ON mp.old_int_id = vwp.watering_plan_id;
DROP TABLE vehicle_watering_plans;
ALTER TABLE vehicle_watering_plans_new RENAME TO vehicle_watering_plans;
ALTER INDEX vehicle_watering_plans_new_pkey
    RENAME TO vehicle_watering_plans_pkey;

-- == user_watering_plans (user_id is already uuid; watering_plan_id swaps int->uuid) ==
CREATE TABLE user_watering_plans_new (
    user_id          uuid NOT NULL,
    watering_plan_id uuid NOT NULL,
    PRIMARY KEY (user_id, watering_plan_id)
);
INSERT INTO user_watering_plans_new (user_id, watering_plan_id)
    SELECT uwp.user_id, mp.new_uuid
      FROM user_watering_plans uwp
      JOIN map_watering_plans mp ON mp.old_int_id = uwp.watering_plan_id;
DROP TABLE user_watering_plans;
ALTER TABLE user_watering_plans_new RENAME TO user_watering_plans;
ALTER INDEX user_watering_plans_new_pkey
    RENAME TO user_watering_plans_pkey;

-- ---------------------------------------------------------------------------
-- 5) Re-establish every foreign key. Order doesn't matter — all data
--    references valid rows in the new tables.
-- ---------------------------------------------------------------------------

ALTER TABLE trees
    ADD CONSTRAINT trees_tree_cluster_id_fkey
    FOREIGN KEY (tree_cluster_id) REFERENCES tree_clusters(id);
ALTER TABLE trees
    ADD CONSTRAINT trees_sensor_id_fkey
    FOREIGN KEY (sensor_id) REFERENCES sensors(id);

ALTER TABLE tree_clusters
    ADD CONSTRAINT tree_clusters_region_id_fkey
    FOREIGN KEY (region_id) REFERENCES regions(id);

ALTER TABLE sensors
    ADD CONSTRAINT sensors_model_id_fkey
    FOREIGN KEY (model_id) REFERENCES sensor_models(id);

ALTER TABLE sensor_data
    ADD CONSTRAINT sensor_data_sensor_id_fkey
    FOREIGN KEY (sensor_id) REFERENCES sensors(id) ON DELETE CASCADE;

ALTER TABLE sensor_model_abilities
    ADD CONSTRAINT sensor_model_abilities_sensor_model_id_fkey
    FOREIGN KEY (sensor_model_id) REFERENCES sensor_models(id) ON DELETE CASCADE;
ALTER TABLE sensor_model_abilities
    ADD CONSTRAINT sensor_model_abilities_sensor_ability_id_fkey
    FOREIGN KEY (sensor_ability_id) REFERENCES sensor_abilities(id);

ALTER TABLE sensor_data_ability_values
    ADD CONSTRAINT sensor_data_ability_values_sensor_data_id_fkey
    FOREIGN KEY (sensor_data_id) REFERENCES sensor_data(id) ON DELETE CASCADE;
ALTER TABLE sensor_data_ability_values
    ADD CONSTRAINT sensor_data_ability_values_sensor_model_ability_id_fkey
    FOREIGN KEY (sensor_model_ability_id) REFERENCES sensor_model_abilities(id);

ALTER TABLE user_watering_plans
    ADD CONSTRAINT user_watering_plans_watering_plan_id_fkey
    FOREIGN KEY (watering_plan_id) REFERENCES watering_plans(id) ON DELETE CASCADE;

ALTER TABLE vehicle_watering_plans
    ADD CONSTRAINT vehicle_watering_plans_vehicle_id_fkey
    FOREIGN KEY (vehicle_id) REFERENCES vehicles(id) ON DELETE CASCADE;
ALTER TABLE vehicle_watering_plans
    ADD CONSTRAINT vehicle_watering_plans_watering_plan_id_fkey
    FOREIGN KEY (watering_plan_id) REFERENCES watering_plans(id) ON DELETE CASCADE;

ALTER TABLE tree_cluster_watering_plans
    ADD CONSTRAINT tree_cluster_watering_plans_tree_cluster_id_fkey
    FOREIGN KEY (tree_cluster_id) REFERENCES tree_clusters(id) ON DELETE CASCADE;
ALTER TABLE tree_cluster_watering_plans
    ADD CONSTRAINT tree_cluster_watering_plans_watering_plan_id_fkey
    FOREIGN KEY (watering_plan_id) REFERENCES watering_plans(id) ON DELETE CASCADE;

ALTER TABLE sensor_lorawan
    ADD CONSTRAINT sensor_lorawan_id_fkey
    FOREIGN KEY (id) REFERENCES sensors(id) ON DELETE CASCADE;

-- ---------------------------------------------------------------------------
-- 6) Drop the orphaned SERIAL sequences from the original schema.
-- ---------------------------------------------------------------------------

DROP SEQUENCE IF EXISTS regions_id_seq;
DROP SEQUENCE IF EXISTS tree_clusters_id_seq;
DROP SEQUENCE IF EXISTS trees_id_seq;
DROP SEQUENCE IF EXISTS vehicles_id_seq;
DROP SEQUENCE IF EXISTS watering_plans_id_seq;
DROP SEQUENCE IF EXISTS sensor_models_id_seq;
DROP SEQUENCE IF EXISTS sensor_data_id_seq;
DROP SEQUENCE IF EXISTS sensor_abilities_id_seq;
DROP SEQUENCE IF EXISTS sensor_model_abilities_id_seq;
DROP SEQUENCE IF EXISTS sensors_id_seq;

COMMIT;
