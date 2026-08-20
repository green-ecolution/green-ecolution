-- Adapts a freshly restored staging pg_dump to the local dev setup.
--
-- Runs directly after backend/seeds/01_organizations.sql inside one
-- transaction (see scripts/import-staging-dump.sh). The dump brings the
-- staging organization tree and the staging Keycloak identities, neither of
-- which exists locally: every resource is re-pointed at the seeded TBZ org,
-- the staging org subtree is dropped, and the profiles/role assignments of the
-- imported dev realm users (.docker/infra/keycloak/green-ecolution-realm.json)
-- replace the staging ones.
--
-- Dev ids (kept in sync with backend/seeds/01_organizations.sql):
--   root      01980000-0000-7000-8000-000000000001  (created by the RBAC migration)
--   TBZ       01980000-0000-7000-8000-000000000002
--   Extern A  01980000-0000-7000-8000-000000000003
--   Extern B  01980000-0000-7000-8000-000000000004

SET search_path TO public;

-- 1. All imported resources move into the seeded TBZ org. The staging dump
--    splits them between its root and its own TBZ org; root is an ancestor of
--    TBZ, so ge.admin still sees everything while tbz.* users get a complete
--    data set instead of trees without sensors or vehicles.
UPDATE depots         SET organization_id = '01980000-0000-7000-8000-000000000002';
UPDATE sensors        SET organization_id = '01980000-0000-7000-8000-000000000002';
UPDATE tree_clusters  SET organization_id = '01980000-0000-7000-8000-000000000002';
UPDATE trees          SET organization_id = '01980000-0000-7000-8000-000000000002';
UPDATE vehicles       SET organization_id = '01980000-0000-7000-8000-000000000002';
UPDATE watering_plans SET organization_id = '01980000-0000-7000-8000-000000000002';

-- 2. Drop the staging user layer. Those uuids are Keycloak ids from the
--    staging realm; locally they resolve to nothing and show up as ghost users.
UPDATE organizations SET contact_person_id = NULL;
DELETE FROM user_watering_plans;
DELETE FROM role_assignments;
DELETE FROM user_profiles;

-- 3. Drop every organization the dump brought that is not part of the dev
--    setup, plus its roles. Leaf-first, because organizations_parent_id_fkey
--    is ON DELETE RESTRICT and fires per row.
DELETE FROM roles
 WHERE organization_id IS NOT NULL
   AND organization_id NOT IN (
     '01980000-0000-7000-8000-000000000001',
     '01980000-0000-7000-8000-000000000002',
     '01980000-0000-7000-8000-000000000003',
     '01980000-0000-7000-8000-000000000004'
   );

DO $$
BEGIN
  LOOP
    DELETE FROM organizations o
     WHERE o.id NOT IN (
             '01980000-0000-7000-8000-000000000001',
             '01980000-0000-7000-8000-000000000002',
             '01980000-0000-7000-8000-000000000003',
             '01980000-0000-7000-8000-000000000004'
           )
       AND NOT EXISTS (SELECT 1 FROM organizations c WHERE c.parent_id = o.id);
    EXIT WHEN NOT FOUND;
  END LOOP;
END $$;

-- 4. Profiles and role assignments for the dev realm users. Mirrors
--    backend/seeds/99_demo_data.sql; username = password.
INSERT INTO user_profiles (id, status, driving_licenses, organization_id) VALUES
  -- Root
  ('01980000-0000-7000-8001-000000000001'::uuid, 'available', ARRAY['B','C']::driving_license[],            '01980000-0000-7000-8000-000000000001'),
  -- TBZ
  ('01980000-0000-7000-8001-000000000011'::uuid, 'available', ARRAY['B','C']::driving_license[],            '01980000-0000-7000-8000-000000000002'),
  ('01980000-0000-7000-8001-000000000012'::uuid, 'available', ARRAY['B']::driving_license[],                '01980000-0000-7000-8000-000000000002'),
  ('01980000-0000-7000-8001-000000000013'::uuid, 'available', ARRAY['B']::driving_license[],                '01980000-0000-7000-8000-000000000002'),
  ('01980000-0000-7000-8001-000000000014'::uuid, 'available', ARRAY['B','BE','C','CE']::driving_license[],  '01980000-0000-7000-8000-000000000002'),
  ('01980000-0000-7000-8001-000000000015'::uuid, 'available', ARRAY['B']::driving_license[],                '01980000-0000-7000-8000-000000000002'),
  -- Extern A
  ('01980000-0000-7000-8001-000000000021'::uuid, 'available', ARRAY['B','C']::driving_license[],            '01980000-0000-7000-8000-000000000003'),
  ('01980000-0000-7000-8001-000000000022'::uuid, 'available', ARRAY['B']::driving_license[],                '01980000-0000-7000-8000-000000000003'),
  ('01980000-0000-7000-8001-000000000023'::uuid, 'available', ARRAY['B']::driving_license[],                '01980000-0000-7000-8000-000000000003'),
  ('01980000-0000-7000-8001-000000000024'::uuid, 'available', ARRAY['B','BE','C','CE']::driving_license[],  '01980000-0000-7000-8000-000000000003'),
  ('01980000-0000-7000-8001-000000000025'::uuid, 'available', ARRAY['B']::driving_license[],                '01980000-0000-7000-8000-000000000003'),
  -- Extern B
  ('01980000-0000-7000-8001-000000000031'::uuid, 'available', ARRAY['B','C']::driving_license[],            '01980000-0000-7000-8000-000000000004'),
  ('01980000-0000-7000-8001-000000000032'::uuid, 'available', ARRAY['B']::driving_license[],                '01980000-0000-7000-8000-000000000004'),
  ('01980000-0000-7000-8001-000000000033'::uuid, 'available', ARRAY['B']::driving_license[],                '01980000-0000-7000-8000-000000000004'),
  ('01980000-0000-7000-8001-000000000034'::uuid, 'available', ARRAY['B','BE','C','CE']::driving_license[],  '01980000-0000-7000-8000-000000000004'),
  ('01980000-0000-7000-8001-000000000035'::uuid, 'available', ARRAY['B']::driving_license[],                '01980000-0000-7000-8000-000000000004');

INSERT INTO role_assignments (user_id, role_id) VALUES
  ('01980000-0000-7000-8001-000000000001'::uuid, '01980000-0000-7000-8000-0000000000b1'::uuid),
  ('01980000-0000-7000-8001-000000000011'::uuid, '01980000-0000-7000-8000-0000000000c1'::uuid),
  ('01980000-0000-7000-8001-000000000012'::uuid, '01980000-0000-7000-8000-0000000000c2'::uuid),
  ('01980000-0000-7000-8001-000000000013'::uuid, '01980000-0000-7000-8000-0000000000c3'::uuid),
  ('01980000-0000-7000-8001-000000000014'::uuid, '01980000-0000-7000-8000-0000000000c4'::uuid),
  ('01980000-0000-7000-8001-000000000015'::uuid, '01980000-0000-7000-8000-0000000000c5'::uuid),
  ('01980000-0000-7000-8001-000000000021'::uuid, '01980000-0000-7000-8000-0000000000d1'::uuid),
  ('01980000-0000-7000-8001-000000000022'::uuid, '01980000-0000-7000-8000-0000000000d2'::uuid),
  ('01980000-0000-7000-8001-000000000023'::uuid, '01980000-0000-7000-8000-0000000000d3'::uuid),
  ('01980000-0000-7000-8001-000000000024'::uuid, '01980000-0000-7000-8000-0000000000d4'::uuid),
  ('01980000-0000-7000-8001-000000000025'::uuid, '01980000-0000-7000-8000-0000000000d5'::uuid),
  ('01980000-0000-7000-8001-000000000031'::uuid, '01980000-0000-7000-8000-0000000000e1'::uuid),
  ('01980000-0000-7000-8001-000000000032'::uuid, '01980000-0000-7000-8000-0000000000e2'::uuid),
  ('01980000-0000-7000-8001-000000000033'::uuid, '01980000-0000-7000-8000-0000000000e3'::uuid),
  ('01980000-0000-7000-8001-000000000034'::uuid, '01980000-0000-7000-8000-0000000000e4'::uuid),
  ('01980000-0000-7000-8001-000000000035'::uuid, '01980000-0000-7000-8000-0000000000e5'::uuid);

-- 5. tbz.routen owns every imported watering plan.
INSERT INTO user_watering_plans (user_id, watering_plan_id)
SELECT '01980000-0000-7000-8001-000000000014'::uuid, id FROM watering_plans;

-- 6. Hand the first four plan-free clusters (by name) to the external sub-orgs
--    so cross-org denial is testable. Plan-bound clusters stay with TBZ,
--    otherwise a TBZ plan would reference clusters its owner cannot read.
WITH candidate AS (
  SELECT c.id, row_number() OVER (ORDER BY c.name, c.id) AS rn
    FROM tree_clusters c
   WHERE NOT c.archived
     AND NOT EXISTS (
       SELECT 1 FROM tree_cluster_watering_plans p WHERE p.tree_cluster_id = c.id
     )
), target AS (
  SELECT id,
         CASE WHEN rn <= 2 THEN '01980000-0000-7000-8000-000000000003'::uuid
              ELSE '01980000-0000-7000-8000-000000000004'::uuid
         END AS organization_id
    FROM candidate
   WHERE rn <= 4
)
UPDATE tree_clusters c
   SET organization_id = t.organization_id
  FROM target t
 WHERE c.id = t.id;

UPDATE trees t
   SET organization_id = c.organization_id
  FROM tree_clusters c
 WHERE t.tree_cluster_id = c.id
   AND c.organization_id IN (
     '01980000-0000-7000-8000-000000000003',
     '01980000-0000-7000-8000-000000000004'
   );

UPDATE sensors s
   SET organization_id = t.organization_id
  FROM trees t
 WHERE t.sensor_id = s.id
   AND t.organization_id IN (
     '01980000-0000-7000-8000-000000000003',
     '01980000-0000-7000-8000-000000000004'
   );
