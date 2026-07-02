-- vehicle_watering_plans encoded the transporter/trailer roles only
-- positionally (via ARRAY_AGG(DISTINCT vehicle_id), i.e. uuid sort order), so
-- a trailer whose uuid sorts before the transporter's was silently decoded as
-- the transporter. Store the role explicitly instead.
--
-- `vehicle_type` is reused as the role type: a vehicle's role on a plan is
-- exactly its type today (the domain assigns transporter/trailer slots).

ALTER TABLE vehicle_watering_plans ADD COLUMN role vehicle_type;

UPDATE vehicle_watering_plans vwp
   SET role = v.type
  FROM vehicles v
 WHERE v.id = vwp.vehicle_id;

-- Defensive: if a plan somehow holds two vehicles of the same type, keep one
-- deterministically so the unique constraint below can be created.
DELETE FROM vehicle_watering_plans vwp
 WHERE EXISTS (
        SELECT 1
          FROM vehicle_watering_plans other
         WHERE other.watering_plan_id = vwp.watering_plan_id
           AND other.role = vwp.role
           AND other.vehicle_id > vwp.vehicle_id
   );

ALTER TABLE vehicle_watering_plans ALTER COLUMN role SET NOT NULL;

-- At most one vehicle per role per plan. Also provides the previously missing
-- index with leading watering_plan_id for the join in plan reads.
ALTER TABLE vehicle_watering_plans
    ADD CONSTRAINT vehicle_watering_plans_plan_role_unique
    UNIQUE (watering_plan_id, role);
