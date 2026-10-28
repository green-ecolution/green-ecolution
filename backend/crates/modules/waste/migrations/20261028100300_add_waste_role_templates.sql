-- Delivered role template for waste collection, copied into every existing
-- organization. New organizations receive it through the usual template copy.
-- An organization that already owns a role of that name keeps its own and gets
-- no copy; failing here would stop the migration and with it the server start.
INSERT INTO roles (id, organization_id, name, description, permissions, template_key)
VALUES (gen_random_uuid(), NULL, 'Abfallsammlung', 'Behälter und Leerungstouren',
  ARRAY['collection_plan:create','collection_plan:delete','collection_plan:read','collection_plan:update',
        'region:read','sensor:read','start_point:read','vehicle:read',
        'waste_bin:create','waste_bin:delete','waste_bin:read','waste_bin:update'],
  'waste_collection')
ON CONFLICT (organization_id, name) DO NOTHING;

INSERT INTO roles (id, organization_id, name, description, permissions, template_key)
SELECT gen_random_uuid(), o.id, t.name, t.description, t.permissions, t.template_key
FROM organizations o
CROSS JOIN roles t
WHERE t.organization_id IS NULL AND t.template_key = 'waste_collection'
ON CONFLICT (organization_id, name) DO NOTHING;

-- template_key survives a permission edit, so a copy only counts as unedited
-- while its permission set still equals the template's. The template sets are
-- taken before the templates themselves are extended.
WITH template AS (
  SELECT template_key, (SELECT array_agg(DISTINCT p ORDER BY p) FROM unnest(permissions) AS p) AS permissions
  FROM roles
  WHERE organization_id IS NULL AND template_key IN ('administrator', 'observer')
)
UPDATE roles r
SET permissions = (
  SELECT array_agg(DISTINCT p ORDER BY p)
  FROM unnest(r.permissions || CASE r.template_key
    WHEN 'administrator' THEN ARRAY['waste_bin:create','waste_bin:delete','waste_bin:read','waste_bin:update',
                                    'collection_plan:create','collection_plan:delete','collection_plan:read','collection_plan:update']
    ELSE ARRAY['waste_bin:read','collection_plan:read']
  END) AS p)
FROM template t
WHERE r.template_key = t.template_key
  AND (r.organization_id IS NULL
       OR (SELECT array_agg(DISTINCT p ORDER BY p) FROM unnest(r.permissions) AS p) = t.permissions);
