-- Identifies the pass holding the lease: writes of a pass whose lease expired
-- and was taken over by another replica are dropped.
ALTER TABLE data_export_targets ADD COLUMN lease_token UUID;
