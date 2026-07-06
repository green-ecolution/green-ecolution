ALTER TABLE sensors ADD COLUMN activated_at TIMESTAMP;

-- only NULL-vs-not-NULL is load-bearing; updated_at merely approximates the activation time
UPDATE sensors SET activated_at = updated_at WHERE status <> 'prepared';

ALTER TABLE sensors DROP COLUMN status;
DROP TYPE sensor_status;
