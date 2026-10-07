-- Short-lived proof that a user opened a plugin's view. Only the SHA-256 of
-- the ticket is stored; expired rows are purged whenever the plugin gets a new
-- ticket, so no cleanup job is needed.
CREATE TABLE plugin_view_tickets (
    token_hash        TEXT PRIMARY KEY,
    plugin_id         UUID NOT NULL REFERENCES plugins (id) ON DELETE CASCADE,
    user_id           UUID NOT NULL,
    user_display_name TEXT NOT NULL,
    expires_at        TIMESTAMPTZ NOT NULL,
    redeemed_at       TIMESTAMPTZ
);

CREATE INDEX plugin_view_tickets_plugin_expiry ON plugin_view_tickets (plugin_id, expires_at);
