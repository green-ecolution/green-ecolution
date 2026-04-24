-- name: GetAllTrees :many
SELECT t.*
FROM trees t
WHERE
    (COALESCE(array_length(@watering_status::TEXT[], 1), 0) = 0
        OR t.watering_status = ANY((@watering_status::TEXT[])::watering_status[]))
  AND (COALESCE(@provider, '') = '' OR t.provider = @provider)
  AND (COALESCE(array_length(@years::INTEGER[], 1), 0) = 0 OR t.planting_year = ANY(@years::INTEGER[]))
  AND (
    sqlc.narg('hasCluster')::BOOLEAN IS NULL
    OR (t.tree_cluster_id IS NOT NULL) = sqlc.narg('hasCluster')::BOOLEAN
      )
  AND (sqlc.narg('tree_cluster_id')::INTEGER IS NULL OR t.tree_cluster_id = sqlc.narg('tree_cluster_id'))
  AND (sqlc.narg('sensor_id')::TEXT IS NULL OR t.sensor_id = sqlc.narg('sensor_id'))
  AND (COALESCE(array_length(@ids::INTEGER[], 1), 0) = 0 OR t.id = ANY(@ids::INTEGER[]))
  ORDER BY t.number ASC
    LIMIT $1 OFFSET $2;

-- name: GetAllTreesCount :one
SELECT COUNT(*)
FROM trees t
WHERE
    (COALESCE(array_length(@watering_status::TEXT[], 1), 0) = 0
        OR t.watering_status = ANY((@watering_status::TEXT[])::watering_status[]))
  AND (COALESCE(@provider, '') = '' OR t.provider = @provider)
  AND (COALESCE(array_length(@years::INTEGER[], 1), 0) = 0 OR t.planting_year = ANY(@years::INTEGER[]))
  AND (
    sqlc.narg('hasCluster')::BOOLEAN IS NULL
    OR (t.tree_cluster_id IS NOT NULL) = sqlc.narg('hasCluster')::BOOLEAN
      )
  AND (sqlc.narg('tree_cluster_id')::INTEGER IS NULL OR t.tree_cluster_id = sqlc.narg('tree_cluster_id'))
  AND (sqlc.narg('sensor_id')::TEXT IS NULL OR t.sensor_id = sqlc.narg('sensor_id'))
  AND (COALESCE(array_length(@ids::INTEGER[], 1), 0) = 0 OR t.id = ANY(@ids::INTEGER[]));

-- name: GetTreeByID :one
SELECT * FROM trees WHERE id = $1;

-- name: CreateTree :one
INSERT INTO trees (
  tree_cluster_id, sensor_id, planting_year, species, number, description, watering_status, latitude, longitude, provider, additional_informations
) VALUES (
  $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11
) RETURNING id;

-- name: UpdateTree :exec
UPDATE trees SET
  tree_cluster_id = $2,
  sensor_id = $3,
  planting_year = $4,
  species = $5,
  number = $6,
  watering_status = $7,
  description = $8,
  provider = $9,
  additional_informations = $10,
  last_watered = $11
WHERE id = $1;

-- name: SetTreeLocation :exec
UPDATE trees SET
  latitude = $2,
  longitude = $3,
  geometry = ST_SetSRID(ST_MakePoint($2, $3), 4326)
WHERE id = $1;

-- name: UpdateTreeClusterID :exec
UPDATE trees SET tree_cluster_id = $2 WHERE id = ANY($1::int[]);

-- name: UpdateTreeGeometry :exec
UPDATE trees SET
  geometry = ST_GeomFromText($2, 4326)
WHERE id = $1;

-- name: DeleteTree :one
DELETE FROM trees WHERE id = $1 RETURNING id;

-- name: UnlinkTreeClusterID :many
UPDATE trees SET tree_cluster_id = NULL WHERE tree_cluster_id = $1 RETURNING id;

-- name: UnlinkSensorIDFromTrees :exec
UPDATE trees
SET sensor_id = NULL, watering_status = 'unknown'
WHERE sensor_id = $1;

-- name: CalculateGroupedCentroids :one
SELECT ST_AsText(ST_Centroid(ST_Collect(geometry)))::text AS centroid FROM trees WHERE id = ANY($1::int[]);

-- name: FindNearestTrees :many
-- Uses scalar latitude/longitude columns; trees.geometry stores axes swapped.
WITH query_point AS (
  SELECT ST_SetSRID(ST_MakePoint(sqlc.arg(lng)::float8, sqlc.arg(lat)::float8), 4326)::geography AS geog
)
SELECT sqlc.embed(trees),
       ST_Distance(
         ST_SetSRID(ST_MakePoint(trees.longitude, trees.latitude), 4326)::geography,
         (SELECT geog FROM query_point)
       )::float8 AS distance
FROM trees
WHERE ST_Distance(
        ST_SetSRID(ST_MakePoint(trees.longitude, trees.latitude), 4326)::geography,
        (SELECT geog FROM query_point)
      ) <= sqlc.arg(radius)::float8
ORDER BY distance ASC
LIMIT sqlc.arg(max_results)::int;

-- name: GetDistinctPlantingYears :many
SELECT DISTINCT planting_year FROM trees WHERE planting_year IS NOT NULL ORDER BY planting_year ASC;