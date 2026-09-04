-- `unknown` was the Go backend's zero value for this enum and never a real
-- lifecycle state: no aggregate transition writes it and the INSERT names
-- 'planned' explicitly, so only the column default could ever have produced it.
--
-- Postgres cannot drop an enum label, so the type is rebuilt. The UPDATE runs
-- while the old type still carries the label, salvaging any row written by a
-- manual DB edit.

UPDATE watering_plans SET status = 'planned' WHERE status = 'unknown';

ALTER TABLE watering_plans ALTER COLUMN status DROP DEFAULT;

ALTER TYPE watering_plan_status RENAME TO watering_plan_status_old;

CREATE TYPE watering_plan_status AS ENUM ('planned', 'active', 'canceled', 'finished', 'not_completed');

ALTER TABLE watering_plans
    ALTER COLUMN status TYPE watering_plan_status
    USING status::text::watering_plan_status;

ALTER TABLE watering_plans ALTER COLUMN status SET DEFAULT 'planned';

DROP TYPE watering_plan_status_old;
