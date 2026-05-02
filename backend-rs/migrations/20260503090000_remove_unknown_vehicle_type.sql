ALTER TABLE vehicles ALTER COLUMN type DROP DEFAULT;

UPDATE vehicles SET type = 'transporter' WHERE type = 'unknown';

CREATE TYPE vehicle_type_new AS ENUM ('transporter', 'trailer');

ALTER TABLE vehicles
    ALTER COLUMN type TYPE vehicle_type_new
    USING type::text::vehicle_type_new;

DROP TYPE vehicle_type;
ALTER TYPE vehicle_type_new RENAME TO vehicle_type;

ALTER TABLE vehicles ALTER COLUMN type SET DEFAULT 'transporter';
