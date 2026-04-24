-- name: GetAllVehicles :many
SELECT *
FROM vehicles
WHERE
    (COALESCE(@provider, '') = '' OR provider = @provider)
  AND (sqlc.narg('vehicle_type')::vehicle_type IS NULL OR type = sqlc.narg('vehicle_type'))
  AND (
    CASE
      WHEN @only_archived::BOOLEAN = TRUE THEN archived_at IS NOT NULL
      WHEN @with_archived::BOOLEAN = TRUE THEN TRUE
      ELSE archived_at IS NULL
    END
  )
ORDER BY water_capacity DESC
LIMIT $1 OFFSET $2;

-- name: GetAllVehiclesCount :one
SELECT COUNT(*)
FROM vehicles
WHERE
    (COALESCE(@provider, '') = '' OR provider = @provider)
  AND (sqlc.narg('vehicle_type')::vehicle_type IS NULL OR type = sqlc.narg('vehicle_type'))
  AND (
    CASE
      WHEN @only_archived::BOOLEAN = TRUE THEN archived_at IS NOT NULL
      WHEN @with_archived::BOOLEAN = TRUE THEN TRUE
      ELSE archived_at IS NULL
    END
  );

-- name: GetVehicleByID :one
SELECT * FROM vehicles WHERE id = $1;

-- name: GetVehicleByPlate :one
SELECT * FROM vehicles WHERE number_plate = $1;

-- name: CreateVehicle :one
INSERT INTO vehicles (
  number_plate,
  description,
  water_capacity,
  type,
  status,
  model,
  driving_license,
  height,
  length,
  width,
  weight,
  provider,
  additional_informations
) VALUES (
  $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13
) RETURNING id;

-- name: UpdateVehicle :exec
UPDATE vehicles SET
  number_plate = $2,
  description = $3,
  water_capacity = $4,
  type = $5,
  status = $6,
  model = $7,
  driving_license = $8,
  height = $9,
  length = $10,
  width = $11,
  weight = $12,
  provider = $13,
  additional_informations = $14
WHERE id = $1;

-- name: ArchiveVehicle :one
UPDATE vehicles SET archived_at = $2 WHERE id = $1 RETURNING id;

-- name: DeleteVehicle :one
DELETE FROM vehicles WHERE id = $1 RETURNING id;

-- name: GetAllVehiclesWithWateringPlanCount :many
SELECT
    v.number_plate,
    COUNT(vwp.watering_plan_id) AS watering_plan_count
FROM vehicles v
INNER JOIN vehicle_watering_plans vwp ON v.id = vwp.vehicle_id
GROUP BY v.number_plate
ORDER BY watering_plan_count DESC;
