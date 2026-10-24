CREATE TABLE irrigation_settings (
    organization_id       UUID PRIMARY KEY REFERENCES organizations (id) ON DELETE CASCADE,
    water_demand_liters   DOUBLE PRECISION,
    just_watered_ttl_secs BIGINT,
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER update_irrigation_settings_updated_at
BEFORE UPDATE ON irrigation_settings
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

INSERT INTO irrigation_settings (organization_id, water_demand_liters, just_watered_ttl_secs, updated_at)
SELECT organization_id, water_demand_liters, just_watered_ttl_secs, updated_at
FROM organization_settings
WHERE water_demand_liters IS NOT NULL OR just_watered_ttl_secs IS NOT NULL;

ALTER TABLE organization_settings
    DROP COLUMN water_demand_liters,
    DROP COLUMN just_watered_ttl_secs;
