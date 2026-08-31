-- Three enum labels carry a space, and one of them a typo, which makes them
-- unusable as i18n key segments on the client. Renaming the label keeps every
-- stored row valid: Postgres rewrites the label in place, rows are untouched.
--
-- 'not competed' was a typo carried over from the Go backend.

ALTER TYPE watering_status RENAME VALUE 'just watered' TO 'just_watered';
ALTER TYPE vehicle_status RENAME VALUE 'not available' TO 'not_available';
ALTER TYPE watering_plan_status RENAME VALUE 'not competed' TO 'not_completed';
