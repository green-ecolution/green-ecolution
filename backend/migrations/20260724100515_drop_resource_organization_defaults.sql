-- Every writer now sets organization_id explicitly (Tasks 12/13); the
-- backfill default from the scoping migration is no longer needed and would
-- silently mask a writer that forgot to scope a new row.
ALTER TABLE trees          ALTER COLUMN organization_id DROP DEFAULT;
ALTER TABLE tree_clusters  ALTER COLUMN organization_id DROP DEFAULT;
ALTER TABLE sensors        ALTER COLUMN organization_id DROP DEFAULT;
ALTER TABLE watering_plans ALTER COLUMN organization_id DROP DEFAULT;
ALTER TABLE vehicles       ALTER COLUMN organization_id DROP DEFAULT;
ALTER TABLE depots         ALTER COLUMN organization_id DROP DEFAULT;
