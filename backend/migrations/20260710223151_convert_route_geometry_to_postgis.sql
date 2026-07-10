-- Route polylines belong in PostGIS like every other geometry in the schema;
-- JSONB was a shortcut when route persistence first landed. Stored arrays are
-- [lat, lon] pairs; a LineString needs at least two points, shorter tracks
-- collapse to NULL ("no route").
ALTER TABLE watering_plans ADD COLUMN route_geom GEOMETRY(LINESTRING, 4326);

UPDATE watering_plans
SET route_geom = ST_SetSRID(ST_MakeLine(ARRAY(
        SELECT ST_MakePoint((pt->>1)::float8, (pt->>0)::float8)
        FROM jsonb_array_elements(route_geometry) WITH ORDINALITY AS t(pt, ord)
        ORDER BY ord
    )), 4326)
WHERE route_geometry IS NOT NULL AND jsonb_array_length(route_geometry) >= 2;

ALTER TABLE watering_plans DROP COLUMN route_geometry;
ALTER TABLE watering_plans RENAME COLUMN route_geom TO route_geometry;
