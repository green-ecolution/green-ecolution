-- Deleting the roles cascades any remaining assignments to them.
-- Delete child orgs (Extern A/B) before TBZ: parent_id FK is ON DELETE RESTRICT.
DELETE FROM roles WHERE organization_id IN (
    '01980000-0000-7000-8000-000000000002',
    '01980000-0000-7000-8000-000000000003',
    '01980000-0000-7000-8000-000000000004'
);

DELETE FROM organizations WHERE id IN (
    '01980000-0000-7000-8000-000000000003',
    '01980000-0000-7000-8000-000000000004'
);

DELETE FROM organizations WHERE id = '01980000-0000-7000-8000-000000000002';
