-- A viewport may now leave its bounding box unset, which means the map is not
-- penned in at all. The centre stays mandatory whenever a viewport is stored:
-- the map has to open somewhere, and a box without a centre would say where a
-- user may pan but not where they start.
ALTER TABLE organization_settings
    DROP CONSTRAINT organization_settings_map_all_or_nothing;

ALTER TABLE organization_settings
    ADD CONSTRAINT organization_settings_map_centre_and_optional_bbox CHECK (
        num_nonnulls(map_center_lat, map_center_lng) IN (0, 2)
        AND num_nonnulls(map_bbox_sw_lat, map_bbox_sw_lng,
                         map_bbox_ne_lat, map_bbox_ne_lng) IN (0, 4)
        AND (map_bbox_sw_lat IS NULL OR map_center_lat IS NOT NULL)
    );
