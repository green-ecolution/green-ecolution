-- One export target (a SensorThings server) per organization; it serves the
-- organization's subtree. The sync columns (cursor, lease, backoff, last run)
-- are written by the export worker.
CREATE TABLE data_export_targets (
    organization_id   UUID PRIMARY KEY REFERENCES organizations (id) ON DELETE CASCADE,
    kind              TEXT        NOT NULL DEFAULT 'sensorthings' CHECK (kind IN ('sensorthings')),
    enabled           BOOLEAN     NOT NULL,
    base_url          TEXT        NOT NULL CHECK (length(base_url) <= 2048),
    auth_kind         TEXT        NOT NULL
        CHECK (auth_kind IN ('none', 'basic', 'oauth2_client_credentials')),
    auth_principal    TEXT CHECK (length(auth_principal) <= 256),
    -- Plaintext like sensor_lorawan.app_key; encryption at rest is a separate topic.
    auth_secret       TEXT CHECK (length(auth_secret) <= 1024),
    auth_token_url    TEXT CHECK (length(auth_token_url) <= 2048),
    auth_scope        TEXT CHECK (length(auth_scope) <= 256),
    activated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    cursor_reading_id UUID,
    leased_until      TIMESTAMPTZ,
    next_attempt_at   TIMESTAMPTZ,
    failure_count     INTEGER     NOT NULL DEFAULT 0,
    last_run_at       TIMESTAMPTZ,
    last_success_at   TIMESTAMPTZ,
    last_error        TEXT,
    last_run_stats    JSONB,
    CONSTRAINT data_export_targets_auth_shape CHECK (
        (auth_kind = 'none'
            AND auth_principal IS NULL AND auth_secret IS NULL
            AND auth_token_url IS NULL AND auth_scope IS NULL)
        OR (auth_kind = 'basic'
            AND auth_principal IS NOT NULL AND auth_secret IS NOT NULL
            AND auth_token_url IS NULL AND auth_scope IS NULL)
        OR (auth_kind = 'oauth2_client_credentials'
            AND auth_principal IS NOT NULL AND auth_secret IS NOT NULL
            AND auth_token_url IS NOT NULL)
    )
);
