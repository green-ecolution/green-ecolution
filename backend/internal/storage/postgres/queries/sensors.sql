-- name: GetAllSensors :many
SELECT * FROM sensors 
WHERE (COALESCE(@provider, '') = '' OR provider = @provider)
ORDER BY id 
LIMIT $1 OFFSET $2;

-- name: GetAllSensorsCount :one
SELECT COUNT(*) FROM sensors
WHERE (COALESCE(@provider, '') = '' OR provider = @provider);

-- name: GetSensorByID :one
SELECT * FROM sensors WHERE id = $1;

-- name: GetSensorByStatus :many
SELECT * FROM sensors WHERE status = $1;

-- name: GetAllSensorDataByID :many
SELECT *
FROM sensor_data
WHERE sensor_id = $1
ORDER BY created_at DESC;

-- name: GetLatestSensorDataByID :one
SELECT *
FROM sensor_data
WHERE sensor_id = $1
ORDER BY created_at DESC
LIMIT 1;

-- name: CreateSensor :one
INSERT INTO sensors (
    id, status, latitude, longitude, provider, additional_informations
) VALUES (
  $1, $2, $3, $4, $5, $6
) RETURNING id;

-- name: UpdateSensor :exec
UPDATE sensors SET
  status = $2,
  provider = $3,
  additional_informations = $4
WHERE id = $1;

-- name: SetSensorLocation :exec
UPDATE sensors SET
    latitude = $2,
    longitude = $3,
    geometry = ST_SetSRID(ST_MakePoint($2, $3), 4326)
WHERE id = $1;

-- name: InsertSensorData :exec
INSERT INTO sensor_data (
  sensor_id, data
) VALUES (
  $1, $2
) RETURNING id;

-- name: DeleteSensor :exec
DELETE FROM sensors WHERE id = $1;
