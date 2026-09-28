-- The map's cluster outlines were derived from every tree on each request
-- (about 1 s at 500k trees). They are now stored and refreshed whenever a
-- cluster's trees or their positions change.
ALTER TABLE tree_clusters ADD COLUMN boundary geometry(Polygon, 4326);

-- Same shape the read path used to compute. The 10 m margin must match
-- CLUSTER_BOUNDARY_BUFFER_METERS in pg_cluster.rs.
--
-- The trigger is paused so the backfill does not stamp every cluster as
-- edited just now.
ALTER TABLE tree_clusters DISABLE TRIGGER update_tree_clusters_updated_at;

UPDATE tree_clusters tc
SET boundary = hull.boundary
FROM (
    SELECT tree_cluster_id,
           ST_Buffer(ST_ConvexHull(ST_Collect(geometry))::geography, 10)::geometry AS boundary
    FROM trees
    WHERE tree_cluster_id IS NOT NULL
      AND geometry IS NOT NULL
    GROUP BY tree_cluster_id
) hull
WHERE hull.tree_cluster_id = tc.id;

ALTER TABLE tree_clusters ENABLE TRIGGER update_tree_clusters_updated_at;
