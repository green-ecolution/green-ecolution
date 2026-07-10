-- Rebuild depots to align coordinate columns with the rest of the schema
-- (latitude/longitude instead of lat/lon, plus a PostGIS point) and to get
-- the desired column order, which ALTER TABLE cannot change.
CREATE TABLE depots_new (
    id             UUID PRIMARY KEY,
    created_at     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    name           TEXT NOT NULL,
    watering_point BOOLEAN NOT NULL DEFAULT FALSE,
    is_default     BOOLEAN NOT NULL DEFAULT FALSE,
    latitude       DOUBLE PRECISION NOT NULL,
    longitude      DOUBLE PRECISION NOT NULL,
    geometry       GEOMETRY(Point, 4326) NOT NULL
);

INSERT INTO depots_new (id, created_at, updated_at, name, watering_point, is_default, latitude, longitude, geometry)
SELECT id, created_at, updated_at, name, watering_point, is_default, lat, lon,
       ST_SetSRID(ST_MakePoint(lon, lat), 4326)
FROM depots;

DROP TABLE depots;
ALTER TABLE depots_new RENAME TO depots;

CREATE UNIQUE INDEX depots_single_default
    ON depots (is_default) WHERE is_default;

CREATE TRIGGER update_depots_updated_at
BEFORE UPDATE ON depots
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
