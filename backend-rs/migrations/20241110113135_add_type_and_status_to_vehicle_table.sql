CREATE TYPE vehicle_status AS ENUM ('active', 'available', 'not available', 'unknown');
CREATE TYPE vehicle_type AS ENUM ('transporter', 'trailer', 'unknown');

ALTER TABLE vehicles
ADD COLUMN type vehicle_type NOT NULL DEFAULT 'unknown',
ADD COLUMN status vehicle_status NOT NULL DEFAULT 'unknown';

