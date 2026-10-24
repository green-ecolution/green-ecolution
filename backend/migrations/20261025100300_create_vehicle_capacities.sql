CREATE TABLE vehicle_capacities (
    vehicle_id UUID NOT NULL REFERENCES vehicles (id) ON DELETE CASCADE,
    kind       TEXT NOT NULL CHECK (kind IN ('water', 'cargo_volume')),
    liters     DOUBLE PRECISION NOT NULL CHECK (liters >= 0),
    PRIMARY KEY (vehicle_id, kind)
);

INSERT INTO vehicle_capacities (vehicle_id, kind, liters)
SELECT id, 'water', water_capacity FROM vehicles;

ALTER TABLE vehicles DROP COLUMN water_capacity;
