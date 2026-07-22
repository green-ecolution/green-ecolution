CREATE TABLE organizations (
    id         UUID PRIMARY KEY,
    parent_id  UUID REFERENCES organizations (id) ON DELETE RESTRICT,
    name       TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (parent_id, name)
);

CREATE UNIQUE INDEX organizations_single_root ON organizations ((TRUE)) WHERE parent_id IS NULL;

CREATE TRIGGER update_organizations_updated_at
BEFORE UPDATE ON organizations
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE roles (
    id              UUID PRIMARY KEY,
    -- NULL = template (delivered by migration, immutable via API)
    organization_id UUID REFERENCES organizations (id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    description     TEXT,
    permissions     TEXT[] NOT NULL,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE NULLS NOT DISTINCT (organization_id, name)
);

CREATE TRIGGER update_roles_updated_at
BEFORE UPDATE ON roles
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE role_assignments (
    user_id UUID NOT NULL REFERENCES user_profiles (id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles (id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id)
);

ALTER TABLE user_profiles
    ADD COLUMN organization_id UUID REFERENCES organizations (id) ON DELETE RESTRICT;

-- Fixed v7-shaped UUIDs so every environment shares the same seed ids.
INSERT INTO organizations (id, parent_id, name)
VALUES ('01980000-0000-7000-8000-000000000001', NULL, 'Green Ecolution');

INSERT INTO roles (id, organization_id, name, description, permissions) VALUES
('01980000-0000-7000-8000-0000000000a1', NULL, 'Administrator', 'Voller Zugriff auf alle Ressourcen',
    (SELECT array_agg(res || ':' || act ORDER BY res, act)
       FROM unnest(ARRAY['tree','tree_cluster','sensor','watering_plan','vehicle','region','user','organization','role']) AS res
      CROSS JOIN unnest(ARRAY['read','create','update','delete']) AS act)),
('01980000-0000-7000-8000-0000000000a2', NULL, 'Baumpflege', 'Pflege von Bäumen und Baumgruppen',
    ARRAY['tree:read','tree:create','tree:update','tree:delete',
          'tree_cluster:read','tree_cluster:create','tree_cluster:update','tree_cluster:delete',
          'sensor:read','region:read']),
('01980000-0000-7000-8000-0000000000a3', NULL, 'Sensorik', 'Verwaltung der Sensorik',
    ARRAY['sensor:read','sensor:create','sensor:update','sensor:delete','tree:read','tree_cluster:read']),
('01980000-0000-7000-8000-0000000000a4', NULL, 'Routenplanung', 'Planung von Bewässerungsrouten',
    ARRAY['watering_plan:read','watering_plan:create','watering_plan:update','watering_plan:delete',
          'vehicle:read','vehicle:create','vehicle:update','vehicle:delete',
          'tree:read','tree_cluster:read','sensor:read','region:read']),
('01980000-0000-7000-8000-0000000000a5', NULL, 'Beobachter', 'Lesender Zugriff auf alle fachlichen Ressourcen',
    ARRAY['tree:read','tree_cluster:read','sensor:read','watering_plan:read','vehicle:read','region:read']);

INSERT INTO roles (id, organization_id, name, description, permissions)
SELECT m.copy_id::uuid, '01980000-0000-7000-8000-000000000001', t.name, t.description, t.permissions
FROM roles t
JOIN (VALUES
    ('01980000-0000-7000-8000-0000000000a1', '01980000-0000-7000-8000-0000000000b1'),
    ('01980000-0000-7000-8000-0000000000a2', '01980000-0000-7000-8000-0000000000b2'),
    ('01980000-0000-7000-8000-0000000000a3', '01980000-0000-7000-8000-0000000000b3'),
    ('01980000-0000-7000-8000-0000000000a4', '01980000-0000-7000-8000-0000000000b4'),
    ('01980000-0000-7000-8000-0000000000a5', '01980000-0000-7000-8000-0000000000b5')
) AS m(template_id, copy_id) ON t.id = m.template_id::uuid;
