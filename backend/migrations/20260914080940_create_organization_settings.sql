-- Typed, nullable columns instead of a key-value schema: NULL is "inherit",
-- query_as! stays type-checked and an unknown key cannot be stored. The price
-- is one migration per new value, which is acceptable at this count.
CREATE TABLE organization_settings (
    organization_id           UUID PRIMARY KEY REFERENCES organizations (id) ON DELETE CASCADE,
    water_demand_liters       DOUBLE PRECISION,
    just_watered_ttl_secs     BIGINT,
    sensor_offline_after_secs BIGINT,
    defect_streak             INTEGER,
    map_center_lat            DOUBLE PRECISION,
    map_center_lng            DOUBLE PRECISION,
    map_bbox_sw_lat           DOUBLE PRECISION,
    map_bbox_sw_lng           DOUBLE PRECISION,
    map_bbox_ne_lat           DOUBLE PRECISION,
    map_bbox_ne_lng           DOUBLE PRECISION,
    descendants_may_override  BOOLEAN NOT NULL DEFAULT TRUE,
    updated_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- The six map columns are only meaningful together; a half-set viewport
    -- is kept out of the table rather than caught in Rust afterwards.
    CONSTRAINT organization_settings_map_all_or_nothing CHECK (
        num_nonnulls(map_center_lat, map_center_lng, map_bbox_sw_lat,
                     map_bbox_sw_lng, map_bbox_ne_lat, map_bbox_ne_lng) IN (0, 6)
    )
);

COMMENT ON COLUMN organization_settings.descendants_may_override IS
  'FALSE freezes this organization''s values for its whole subtree.';

CREATE TRIGGER update_organization_settings_updated_at
BEFORE UPDATE ON organization_settings
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
