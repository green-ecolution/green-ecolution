ALTER TABLE sensor_data_ability_values
    ADD COLUMN plausible      boolean NOT NULL DEFAULT true,
    ADD COLUMN quality_reason text;

-- Serves the data-quality aggregates, which only ever look for flagged rows.
CREATE INDEX idx_sdav_implausible
    ON sensor_data_ability_values (sensor_data_id)
    WHERE NOT plausible;
