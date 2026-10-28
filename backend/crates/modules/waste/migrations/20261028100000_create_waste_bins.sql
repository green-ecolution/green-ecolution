CREATE TABLE waste_bins (
    id                      UUID PRIMARY KEY,
    number                  TEXT NOT NULL,
    latitude                DOUBLE PRECISION NOT NULL,
    longitude               DOUBLE PRECISION NOT NULL,
    geometry                GEOMETRY(Point, 4326) NOT NULL,
    volume_liters           DOUBLE PRECISION NOT NULL CHECK (volume_liters > 0),
    depth_cm                INTEGER NOT NULL CHECK (depth_cm BETWEEN 1 AND 1000),
    organization_id         UUID NOT NULL REFERENCES organizations (id) ON DELETE RESTRICT,
    provider                TEXT,
    additional_informations JSONB,
    fill_fraction           DOUBLE PRECISION CHECK (fill_fraction BETWEEN 0 AND 1),
    fill_measured_at        TIMESTAMPTZ,
    last_emptied_at         TIMESTAMPTZ,
    updated_at              TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CHECK ((fill_fraction IS NULL) = (fill_measured_at IS NULL))
);

CREATE UNIQUE INDEX waste_bins_number_per_org ON waste_bins (organization_id, number);
CREATE INDEX waste_bins_geometry ON waste_bins USING GIST (geometry);
CREATE INDEX waste_bins_organization_id_idx ON waste_bins (organization_id);

CREATE TRIGGER update_waste_bins_updated_at
BEFORE UPDATE ON waste_bins
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
