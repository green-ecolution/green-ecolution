-- The stored column now carries only availability; `active` is derived from
-- the vehicle's assignment to a running watering plan.
ALTER TABLE vehicles ALTER COLUMN status DROP DEFAULT;

UPDATE vehicles SET status = 'available' WHERE status IN ('unknown', 'active');

CREATE TYPE vehicle_availability AS ENUM ('available', 'not_available');

ALTER TABLE vehicles
    ALTER COLUMN status TYPE vehicle_availability
    USING status::text::vehicle_availability;

DROP TYPE vehicle_status;

ALTER TABLE vehicles RENAME COLUMN status TO availability;
ALTER TABLE vehicles ALTER COLUMN availability SET DEFAULT 'available';
