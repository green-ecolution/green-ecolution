-- Snapshot of the refill stations a plan's computed route visits.
-- Deliberately no FK to depots: renaming or deleting a depot must not
-- rewrite the history of existing plans.
CREATE TABLE watering_plan_refill_points (
    watering_plan_id UUID NOT NULL REFERENCES watering_plans(id) ON DELETE CASCADE,
    position INT NOT NULL,
    name TEXT NOT NULL,
    latitude DOUBLE PRECISION NOT NULL,
    longitude DOUBLE PRECISION NOT NULL,
    geometry GEOMETRY(Point, 4326) NOT NULL,
    PRIMARY KEY (watering_plan_id, position)
);
