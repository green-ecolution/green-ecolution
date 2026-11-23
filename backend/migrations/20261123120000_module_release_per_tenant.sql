-- Module activation moves from "one switch per organization, inherited down
-- the tree" to "released per tenant by the root, switched by the tenant".
-- Afterwards only the root and its direct children (the tenants) hold rows.

ALTER TABLE organization_modules ADD COLUMN released BOOLEAN NOT NULL DEFAULT TRUE;

-- Every tenant gets a row per module with the state it sees today: its own
-- switch, else the root's, else the instance default. A migration cannot read
-- the YAML, so the defaults are named here: irrigation on, everything else off.
WITH root AS (
    SELECT id FROM organizations WHERE parent_id IS NULL
),
tenants AS (
    SELECT o.id FROM organizations o JOIN root r ON o.parent_id = r.id
),
keys AS (
    SELECT 'irrigation' AS module_key, TRUE AS default_enabled
    UNION ALL
    SELECT 'waste', FALSE
    UNION ALL
    SELECT DISTINCT module_key, FALSE
    FROM organization_modules
    WHERE module_key NOT IN ('irrigation', 'waste')
)
INSERT INTO organization_modules (organization_id, module_key, enabled, set_by)
SELECT t.id,
       k.module_key,
       COALESCE(at_root.enabled, k.default_enabled),
       t.id
FROM tenants t
CROSS JOIN keys k
LEFT JOIN organization_modules at_root
       ON at_root.organization_id = (SELECT id FROM root)
      AND at_root.module_key = k.module_key
-- A tenant's own switch stays as it is.
ON CONFLICT (organization_id, module_key) DO NOTHING;

-- Locked is only what the root bindingly switched off for a tenant.
UPDATE organization_modules m
SET released = FALSE
FROM organizations o
WHERE o.id = m.organization_id
  AND o.parent_id = (SELECT id FROM organizations WHERE parent_id IS NULL)
  AND m.set_by = o.parent_id
  AND m.enabled = FALSE;

-- Sub-organizations follow their tenant from now on.
DELETE FROM organization_modules m
USING organizations o
WHERE o.id = m.organization_id
  AND o.parent_id IS NOT NULL
  AND o.parent_id <> (SELECT id FROM organizations WHERE parent_id IS NULL);

ALTER TABLE organization_modules DROP COLUMN set_by;
ALTER TABLE organization_modules ALTER COLUMN released DROP DEFAULT;
