CREATE TABLE waste_settings (
    organization_id        UUID PRIMARY KEY REFERENCES organizations (id) ON DELETE CASCADE,
    full_threshold_percent SMALLINT CHECK (full_threshold_percent BETWEEN 1 AND 100),
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER update_waste_settings_updated_at
BEFORE UPDATE ON waste_settings
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
