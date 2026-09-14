CREATE TABLE plugins (
    id                   UUID PRIMARY KEY,
    slug                 TEXT NOT NULL UNIQUE,
    name                 TEXT NOT NULL,
    description          TEXT,
    frontend_mode        TEXT NOT NULL DEFAULT 'none'
                         CHECK (frontend_mode IN ('none', 'external', 'proxied')),
    frontend_target      TEXT,
    organization_id      UUID NOT NULL REFERENCES organizations (id) ON DELETE RESTRICT,
    permissions          TEXT[] NOT NULL DEFAULT '{}',
    required_permissions TEXT[] NOT NULL DEFAULT '{}',
    enabled              BOOLEAN NOT NULL DEFAULT FALSE,
    key_hash             TEXT,
    last_seen_at         TIMESTAMPTZ,
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT plugins_frontend_target_present
        CHECK ((frontend_mode = 'none') = (frontend_target IS NULL))
);

CREATE TRIGGER update_plugins_updated_at
BEFORE UPDATE ON plugins
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Deliberately tree-specific instead of a polymorphic entity_type: only this
-- shape allows a real foreign key, so a deleted tree leaves no dangling ref.
CREATE TABLE plugin_tree_refs (
    plugin_id   UUID NOT NULL REFERENCES plugins (id) ON DELETE CASCADE,
    external_id TEXT NOT NULL,
    tree_id     UUID NOT NULL REFERENCES trees (id) ON DELETE CASCADE,
    PRIMARY KEY (plugin_id, external_id),
    UNIQUE (plugin_id, tree_id)
);

UPDATE roles
   SET permissions = permissions || ARRAY['plugin:read','plugin:create','plugin:update','plugin:delete']
 WHERE template_key = 'administrator'
   AND NOT permissions @> ARRAY['plugin:read'];
