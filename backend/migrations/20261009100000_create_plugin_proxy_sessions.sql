-- A browser session on a proxied plugin's own host. Only the SHA-256 of the
-- token is stored; expired rows are purged whenever the plugin gets a new
-- session, so no cleanup job is needed.
CREATE TABLE plugin_proxy_sessions (
    token_hash        TEXT PRIMARY KEY,
    plugin_id         UUID NOT NULL REFERENCES plugins (id) ON DELETE CASCADE,
    user_id           UUID NOT NULL,
    user_display_name TEXT NOT NULL,
    expires_at        TIMESTAMPTZ NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX plugin_proxy_sessions_plugin_expires ON plugin_proxy_sessions (plugin_id, expires_at);
