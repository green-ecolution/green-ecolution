-- trees: filters from list/unlink queries plus PostGIS GIST for nearest_trees.
CREATE INDEX IF NOT EXISTS idx_trees_cluster_id
    ON trees(tree_cluster_id) WHERE tree_cluster_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_trees_sensor_id
    ON trees(sensor_id) WHERE sensor_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_trees_provider
    ON trees(provider);
CREATE INDEX IF NOT EXISTS idx_trees_watering_status
    ON trees(watering_status);
CREATE INDEX IF NOT EXISTS idx_trees_geometry
    ON trees USING GIST(geometry);

-- sensors: provider filter and PostGIS for spatial queries.
CREATE INDEX IF NOT EXISTS idx_sensors_provider
    ON sensors(provider);
CREATE INDEX IF NOT EXISTS idx_sensors_geometry
    ON sensors USING GIST(geometry);

-- sensor_data: composite index for "data per sensor newest first" lookups.
CREATE INDEX IF NOT EXISTS idx_sensor_data_sensor_created
    ON sensor_data(sensor_id, created_at DESC);

-- vehicles: filters from list endpoint.
CREATE INDEX IF NOT EXISTS idx_vehicles_provider
    ON vehicles(provider);
CREATE INDEX IF NOT EXISTS idx_vehicles_type
    ON vehicles(type);
CREATE INDEX IF NOT EXISTS idx_vehicles_archived_at
    ON vehicles(archived_at);

-- tree_clusters: filters and FK lookup.
CREATE INDEX IF NOT EXISTS idx_tree_clusters_provider
    ON tree_clusters(provider);
CREATE INDEX IF NOT EXISTS idx_tree_clusters_watering_status
    ON tree_clusters(watering_status);
CREATE INDEX IF NOT EXISTS idx_tree_clusters_region_id
    ON tree_clusters(region_id) WHERE region_id IS NOT NULL;

-- watering_plans: filter + ORDER BY date DESC.
CREATE INDEX IF NOT EXISTS idx_watering_plans_provider
    ON watering_plans(provider);
CREATE INDEX IF NOT EXISTS idx_watering_plans_date
    ON watering_plans(date DESC);

-- regions: PostGIS GIST for ST_Contains lookups in by_point.
CREATE INDEX IF NOT EXISTS idx_regions_geometry
    ON regions USING GIST(geometry);
