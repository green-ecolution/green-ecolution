-- Fix the historical typo in the watering_plan_status enum: the value was
-- created as 'not competed' (see 20241116142300_implement_watering_plans.sql).
-- Forward-only; the project has no down migrations.
ALTER TYPE watering_plan_status RENAME VALUE 'not competed' TO 'not completed';
