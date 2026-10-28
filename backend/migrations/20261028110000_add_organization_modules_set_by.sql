-- The level a switch was set from. Only someone acting for that level or
-- above may change it, so a parent's choice for a sub-unit binds the sub-unit.
-- Existing switches count as the organization's own.
ALTER TABLE organization_modules
    ADD COLUMN set_by UUID REFERENCES organizations(id) ON DELETE CASCADE;

UPDATE organization_modules SET set_by = organization_id;

ALTER TABLE organization_modules ALTER COLUMN set_by SET NOT NULL;
