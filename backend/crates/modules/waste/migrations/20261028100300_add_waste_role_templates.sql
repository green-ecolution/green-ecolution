-- Delivered role template for waste collection, copied into every existing
-- organization. New organizations receive it through the usual template copy.
INSERT INTO roles (id, organization_id, name, description, permissions, template_key)
VALUES (gen_random_uuid(), NULL, 'Abfallsammlung', 'Behälter und Leerungstouren',
  ARRAY['collection_plan:create','collection_plan:delete','collection_plan:read','collection_plan:update',
        'region:read','sensor:read','start_point:read','vehicle:read',
        'waste_bin:create','waste_bin:delete','waste_bin:read','waste_bin:update'],
  'waste_collection');

INSERT INTO roles (id, organization_id, name, description, permissions, template_key)
SELECT gen_random_uuid(), o.id, t.name, t.description, t.permissions, t.template_key
FROM organizations o
CROSS JOIN roles t
WHERE t.organization_id IS NULL AND t.template_key = 'waste_collection';

-- Only roles still matching their template verbatim; an edited role is the organization's own decision.
UPDATE roles
SET permissions = (
  SELECT array_agg(DISTINCT p ORDER BY p)
  FROM unnest(permissions || ARRAY['waste_bin:create','waste_bin:delete','waste_bin:read','waste_bin:update',
                                   'collection_plan:create','collection_plan:delete','collection_plan:read','collection_plan:update']) AS p)
WHERE template_key = 'administrator';

UPDATE roles
SET permissions = (
  SELECT array_agg(DISTINCT p ORDER BY p)
  FROM unnest(permissions || ARRAY['waste_bin:read','collection_plan:read']) AS p)
WHERE template_key = 'observer';
