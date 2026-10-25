ALTER TYPE watering_plan_status RENAME TO operation_plan_status;

ALTER TABLE watering_plans RENAME TO operation_plans;
ALTER TABLE operation_plans
    ADD COLUMN module_key TEXT NOT NULL DEFAULT 'irrigation'
        CHECK (module_key ~ '^[a-z][a-z0-9_]{0,31}$');
ALTER TABLE operation_plans ALTER COLUMN module_key DROP DEFAULT;
ALTER TABLE operation_plans RENAME COLUMN transporter_tank_used TO transporter_capacity_used;
ALTER TABLE operation_plans RENAME COLUMN total_water_required TO total_demand;
CREATE INDEX operation_plans_module_key_idx ON operation_plans (module_key);
ALTER TRIGGER update_watering_plans_updated_at ON operation_plans
    RENAME TO update_operation_plans_updated_at;
ALTER INDEX IF EXISTS idx_watering_plans_provider RENAME TO idx_operation_plans_provider;
ALTER INDEX IF EXISTS idx_watering_plans_date RENAME TO idx_operation_plans_date;
ALTER INDEX IF EXISTS watering_plans_organization_id_idx RENAME TO operation_plans_organization_id_idx;

-- Stops point into module tables the core cannot reference; modules remove
-- them through the plan service instead of a cascade.
ALTER TABLE tree_cluster_watering_plans
    DROP CONSTRAINT tree_cluster_watering_plans_tree_cluster_id_fkey;
ALTER TABLE tree_cluster_watering_plans RENAME TO operation_plan_stops;
ALTER TABLE operation_plan_stops RENAME COLUMN tree_cluster_id TO stop_id;
ALTER TABLE operation_plan_stops RENAME COLUMN watering_plan_id TO plan_id;
ALTER TABLE operation_plan_stops RENAME COLUMN consumed_water TO amount;
ALTER TABLE operation_plan_stops
    ALTER COLUMN amount DROP NOT NULL,
    ALTER COLUMN amount DROP DEFAULT,
    ADD COLUMN completed BOOLEAN NOT NULL DEFAULT FALSE;
UPDATE operation_plan_stops s
SET completed = TRUE
FROM operation_plans p
WHERE p.id = s.plan_id AND p.status = 'finished';
UPDATE operation_plan_stops s
SET amount = NULL
FROM operation_plans p
WHERE p.id = s.plan_id AND p.status <> 'finished';
ALTER INDEX IF EXISTS idx_tree_cluster_watering_plans_plan_id RENAME TO idx_operation_plan_stops_plan_id;

ALTER TABLE vehicle_watering_plans RENAME TO operation_plan_vehicles;
ALTER TABLE operation_plan_vehicles RENAME COLUMN watering_plan_id TO plan_id;

ALTER TABLE user_watering_plans RENAME TO operation_plan_users;
ALTER TABLE operation_plan_users RENAME COLUMN watering_plan_id TO plan_id;
ALTER INDEX IF EXISTS idx_user_watering_plans_plan_id RENAME TO idx_operation_plan_users_plan_id;

ALTER TABLE watering_plan_refill_points RENAME TO operation_plan_station_visits;
ALTER TABLE operation_plan_station_visits RENAME COLUMN watering_plan_id TO plan_id;
