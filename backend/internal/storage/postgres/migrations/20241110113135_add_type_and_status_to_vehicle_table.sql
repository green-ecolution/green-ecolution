-- +goose Up
-- +goose StatementBegin
CREATE TYPE vehicle_status AS ENUM ('active', 'available', 'not available', 'unknown');
CREATE TYPE vehicle_type AS ENUM ('transporter', 'trailer', 'unknown');

ALTER TABLE vehicles
ADD COLUMN type vehicle_type NOT NULL DEFAULT 'unknown',
ADD COLUMN status vehicle_status NOT NULL DEFAULT 'unknown';
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
ALTER TABLE vehicles
DROP COLUMN type,
DROP COLUMN status;

DROP TYPE IF EXISTS vehicle_status;
DROP TYPE IF EXISTS vehicle_type;
-- +goose StatementEnd
