-- Which remote entity (`@iot.id`) a local object, location, sensor, property or
-- datastream became on the organization's export target.
CREATE TABLE data_export_refs (
    organization_id UUID    NOT NULL REFERENCES data_export_targets (organization_id) ON DELETE CASCADE,
    ge_key          TEXT    NOT NULL CHECK (length(ge_key) <= 512),
    entity          TEXT    NOT NULL
        CHECK (entity IN ('thing', 'location', 'sensor', 'observed_property', 'datastream')),
    remote_id       JSONB   NOT NULL,
    content_hash    TEXT,
    last_value      JSONB,
    deleted_marked  BOOLEAN NOT NULL DEFAULT false,
    PRIMARY KEY (organization_id, ge_key)
);
