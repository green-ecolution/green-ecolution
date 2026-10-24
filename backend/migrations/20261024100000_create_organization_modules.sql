-- One row per organization and module the organization switched itself; a
-- missing row means the organization inherits from its parent.
CREATE TABLE organization_modules (
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    module_key TEXT NOT NULL CHECK (module_key ~ '^[a-z][a-z0-9_]{0,31}$'),
    enabled BOOLEAN NOT NULL,
    PRIMARY KEY (organization_id, module_key)
);
