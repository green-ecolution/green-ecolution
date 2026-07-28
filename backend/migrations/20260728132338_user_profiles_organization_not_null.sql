-- Backfill legacy rows to the root organization before enforcing the constraint.
UPDATE user_profiles
SET organization_id = '01980000-0000-7000-8000-000000000001'
WHERE organization_id IS NULL;

ALTER TABLE user_profiles ALTER COLUMN organization_id SET NOT NULL;
