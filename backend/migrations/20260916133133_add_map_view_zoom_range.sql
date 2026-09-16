-- The zoom range belongs to the same decision as the bounding box: an
-- organization either holds its map to a working area, with both a pan limit
-- and a zoom range, or it lifts the limit entirely. The check therefore treats
-- the four box columns and the two zoom columns as one group of six.
ALTER TABLE organization_settings
    ADD COLUMN map_min_zoom SMALLINT,
    ADD COLUMN map_max_zoom SMALLINT;

-- Every row that already carries a box gets the values the application used as
-- hardcoded constants until now, so no existing viewport changes behaviour.
UPDATE organization_settings
   SET map_min_zoom = 13, map_max_zoom = 18
 WHERE map_bbox_sw_lat IS NOT NULL;

ALTER TABLE organization_settings
    DROP CONSTRAINT organization_settings_map_centre_and_optional_bbox;

ALTER TABLE organization_settings
    ADD CONSTRAINT organization_settings_map_centre_and_optional_bounds CHECK (
        num_nonnulls(map_center_lat, map_center_lng) IN (0, 2)
        AND num_nonnulls(map_bbox_sw_lat, map_bbox_sw_lng, map_bbox_ne_lat,
                         map_bbox_ne_lng, map_min_zoom, map_max_zoom) IN (0, 6)
        AND (map_bbox_sw_lat IS NULL OR map_center_lat IS NOT NULL)
        AND (map_min_zoom IS NULL OR map_min_zoom BETWEEN 0 AND 24)
        AND (map_max_zoom IS NULL OR map_max_zoom BETWEEN 0 AND 24)
        AND (map_min_zoom IS NULL OR map_min_zoom <= map_max_zoom)
    );
