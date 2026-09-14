-- The live value is computed with, so it gets typed columns. The history is
-- only ever displayed and never read back into typed code, so a serialized
-- value is the right shape here: it spares the table two more columns per new
-- setting without anyone losing a type check.
CREATE TABLE organization_settings_history (
    id              UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    setting_key     TEXT NOT NULL,
    previous_value  JSONB,
    new_value       JSONB,
    changed_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- A deleted account must not take the record of its changes with it.
    changed_by      UUID REFERENCES user_profiles (id) ON DELETE SET NULL
);

COMMENT ON COLUMN organization_settings_history.new_value IS
  'NULL means the value went back to being inherited.';
COMMENT ON COLUMN organization_settings_history.previous_value IS
  'NULL means the value was inherited before.';

CREATE INDEX organization_settings_history_latest_idx
    ON organization_settings_history (organization_id, setting_key, changed_at DESC, id DESC);
