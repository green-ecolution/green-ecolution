-- +goose Up
-- +goose StatementBegin
ALTER TYPE watering_status ADD VALUE 'just watered';
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
ALTER TYPE watering_status RENAME TO watering_status_old;

CREATE TYPE watering_status AS ENUM ('good', 'moderate', 'bad', 'unknown');

UPDATE tree_clusters SET watering_status = 'unknown' WHERE watering_status = 'just watered';
UPDATE trees SET watering_status = 'unknown' WHERE watering_status = 'just watered';

ALTER TABLE tree_clusters ALTER COLUMN watering_status DROP DEFAULT;
ALTER TABLE trees ALTER COLUMN watering_status DROP DEFAULT;

ALTER TABLE tree_clusters
    ALTER COLUMN watering_status TYPE watering_status USING watering_status::text::watering_status;
ALTER TABLE trees
    ALTER COLUMN watering_status TYPE watering_status USING watering_status::text::watering_status;

ALTER TABLE tree_clusters ALTER COLUMN watering_status SET DEFAULT 'unknown';

DROP TYPE watering_status_old;
-- +goose StatementEnd
