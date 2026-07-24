-- Org ownership on business resources. The DEFAULT backfills existing rows
-- and stays until every writer sets the column explicitly (dropped in a
-- follow-up migration at the end of the scoping work).
ALTER TABLE trees          ADD COLUMN organization_id UUID NOT NULL DEFAULT '01980000-0000-7000-8000-000000000001' REFERENCES organizations(id) ON DELETE RESTRICT;
ALTER TABLE tree_clusters  ADD COLUMN organization_id UUID NOT NULL DEFAULT '01980000-0000-7000-8000-000000000001' REFERENCES organizations(id) ON DELETE RESTRICT;
ALTER TABLE sensors        ADD COLUMN organization_id UUID NOT NULL DEFAULT '01980000-0000-7000-8000-000000000001' REFERENCES organizations(id) ON DELETE RESTRICT;
ALTER TABLE watering_plans ADD COLUMN organization_id UUID NOT NULL DEFAULT '01980000-0000-7000-8000-000000000001' REFERENCES organizations(id) ON DELETE RESTRICT;
ALTER TABLE vehicles       ADD COLUMN organization_id UUID NOT NULL DEFAULT '01980000-0000-7000-8000-000000000001' REFERENCES organizations(id) ON DELETE RESTRICT;
ALTER TABLE depots         ADD COLUMN organization_id UUID NOT NULL DEFAULT '01980000-0000-7000-8000-000000000001' REFERENCES organizations(id) ON DELETE RESTRICT;

CREATE INDEX trees_organization_id_idx          ON trees (organization_id);
CREATE INDEX tree_clusters_organization_id_idx  ON tree_clusters (organization_id);
CREATE INDEX sensors_organization_id_idx        ON sensors (organization_id);
CREATE INDEX watering_plans_organization_id_idx ON watering_plans (organization_id);
CREATE INDEX vehicles_organization_id_idx       ON vehicles (organization_id);
CREATE INDEX depots_organization_id_idx         ON depots (organization_id);

-- one default depot per organization instead of one global default
DROP INDEX depots_single_default;
CREATE UNIQUE INDEX depots_single_default_per_org ON depots (organization_id) WHERE is_default;
