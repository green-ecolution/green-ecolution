-- Dead since the Go backend: the GPX download is rendered on the fly from
-- route_geometry and the response URL is derived, never read from this column.
ALTER TABLE watering_plans DROP COLUMN gpx_url;
