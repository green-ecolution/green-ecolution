-- Dev setup: TBZ organization under the seeded root organization.
-- User memberships and role assignments live in 99_demo_data.sql next to the
-- user_profiles rows they reference.

INSERT INTO organizations (id, parent_id, name)
VALUES ('01980000-0000-7000-8000-000000000002', '01980000-0000-7000-8000-000000000001', 'TBZ');

-- Raw INSERT bypasses OrganizationService, so the template copies the service
-- would create must be seeded too (an org must never exist without its
-- default roles). Same pattern as the root copies in the RBAC migration.
INSERT INTO roles (id, organization_id, name, description, permissions)
SELECT m.copy_id::uuid, '01980000-0000-7000-8000-000000000002', t.name, t.description, t.permissions
FROM roles t
JOIN (VALUES
    ('01980000-0000-7000-8000-0000000000a1', '01980000-0000-7000-8000-0000000000c1'),
    ('01980000-0000-7000-8000-0000000000a2', '01980000-0000-7000-8000-0000000000c2'),
    ('01980000-0000-7000-8000-0000000000a3', '01980000-0000-7000-8000-0000000000c3'),
    ('01980000-0000-7000-8000-0000000000a4', '01980000-0000-7000-8000-0000000000c4'),
    ('01980000-0000-7000-8000-0000000000a5', '01980000-0000-7000-8000-0000000000c5')
) AS m(template_id, copy_id) ON t.id = m.template_id::uuid;
