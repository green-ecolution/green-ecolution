-- name: GetAllRegions :many
SELECT * FROM regions 
ORDER BY id 
LIMIT $1 OFFSET $2;

-- name: GetAllRegionsCount :one
SELECT COUNT(*) FROM regions;

-- name: GetRegionById :one
SELECT * FROM regions WHERE id = $1;

-- name: GetRegionByName :one
SELECT * FROM regions WHERE name = $1;

-- name: CreateRegion :one
INSERT INTO regions (name, geometry) VALUES ($1, ST_GeomFromText($2, 4326)) RETURNING id;

-- name: UpdateRegion :exec
UPDATE regions SET name = $2, geometry = ST_GeomFromText($3, 4326) WHERE id = $1;

-- name: DeleteRegion :exec
DELETE FROM regions WHERE id = $1;

-- name: GetRegionByPoint :one
SELECT * FROM regions WHERE ST_Contains(geometry, ST_GeomFromText($1, 4326));

