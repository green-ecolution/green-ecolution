-- Role templates ship with German names. A client cannot translate them
-- without knowing which delivered role a row is, so record that identity
-- explicitly instead of matching on the name.
--
-- The key is cleared as soon as the name or description is edited (enforced in
-- the aggregate), so a translated label never overrides a name a user chose.

ALTER TABLE roles ADD COLUMN template_key TEXT;

COMMENT ON COLUMN roles.template_key IS
  'Delivered template this role still matches verbatim; NULL once edited or hand-made.';

-- The five templates, plus the copies the RBAC foundation migration created
-- for the root organization. Both are matched by their seeded ids.
UPDATE roles SET template_key = v.key
FROM (VALUES
    ('01980000-0000-7000-8000-0000000000a1', 'administrator'),
    ('01980000-0000-7000-8000-0000000000a2', 'tree_care'),
    ('01980000-0000-7000-8000-0000000000a3', 'sensors'),
    ('01980000-0000-7000-8000-0000000000a4', 'route_planning'),
    ('01980000-0000-7000-8000-0000000000a5', 'observer'),
    ('01980000-0000-7000-8000-0000000000b1', 'administrator'),
    ('01980000-0000-7000-8000-0000000000b2', 'tree_care'),
    ('01980000-0000-7000-8000-0000000000b3', 'sensors'),
    ('01980000-0000-7000-8000-0000000000b4', 'route_planning'),
    ('01980000-0000-7000-8000-0000000000b5', 'observer')
) AS v(id, key)
WHERE roles.id = v.id::uuid;

-- Copies made for organizations created before this column existed carry fresh
-- ids, so they cannot be matched by id. A copy that still has the template's
-- name, description and exact permission set has demonstrably not been edited,
-- which is the same condition the column encodes going forward. Permissions are
-- sorted before comparing, because array equality is order-sensitive.
UPDATE roles r
SET template_key = t.template_key
FROM roles t
WHERE t.organization_id IS NULL
  AND t.template_key IS NOT NULL
  AND r.organization_id IS NOT NULL
  AND r.template_key IS NULL
  AND r.name = t.name
  AND r.description IS NOT DISTINCT FROM t.description
  AND (SELECT array_agg(p ORDER BY p) FROM unnest(r.permissions) AS p)
    = (SELECT array_agg(p ORDER BY p) FROM unnest(t.permissions) AS p);
