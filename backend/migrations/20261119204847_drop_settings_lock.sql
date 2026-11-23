-- The lock on a subtree is gone: every organization may set its own values
-- again. A value a sub-unit had stored while it was locked takes effect with
-- this migration; that is the accepted consequence of dropping the lock.
ALTER TABLE organization_settings DROP COLUMN descendants_may_override;
