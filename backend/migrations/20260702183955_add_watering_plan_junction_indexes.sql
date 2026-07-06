-- The composite PKs of the watering-plan junction tables lead with the other
-- column, so every join/delete by watering_plan_id was a seq scan.
-- vehicle_watering_plans already got its index via the
-- UNIQUE (watering_plan_id, role) constraint; these two were still missing.

CREATE INDEX idx_tree_cluster_watering_plans_plan_id
    ON tree_cluster_watering_plans (watering_plan_id);

CREATE INDEX idx_user_watering_plans_plan_id
    ON user_watering_plans (watering_plan_id);
