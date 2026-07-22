-- Deleting the roles cascades any remaining assignments to them.
DELETE FROM roles WHERE organization_id = '01980000-0000-7000-8000-000000000002';

DELETE FROM organizations WHERE id = '01980000-0000-7000-8000-000000000002';
