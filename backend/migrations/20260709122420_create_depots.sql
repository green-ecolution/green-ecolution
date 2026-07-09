CREATE TABLE depots (
    id             UUID PRIMARY KEY,
    name           TEXT NOT NULL,
    lat            DOUBLE PRECISION NOT NULL,
    lon            DOUBLE PRECISION NOT NULL,
    watering_point BOOLEAN NOT NULL DEFAULT FALSE,
    is_default     BOOLEAN NOT NULL DEFAULT FALSE,
    created_at     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX depots_single_default
    ON depots (is_default) WHERE is_default;

CREATE TRIGGER update_depots_updated_at
BEFORE UPDATE ON depots
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
