ALTER TABLE comments DROP CONSTRAINT comments_subject_type_check;
ALTER TABLE comments
    ADD COLUMN target_module TEXT,
    ADD COLUMN target_kind TEXT;

UPDATE comments
SET subject_type = 'target', target_module = 'irrigation', target_kind = 'cluster'
WHERE subject_type = 'tree_cluster';

UPDATE comments SET subject_type = 'plan' WHERE subject_type = 'watering_plan';

ALTER TABLE comments
    ADD CONSTRAINT comments_subject_shape CHECK (
        (subject_type = 'plan' AND target_module IS NULL AND target_kind IS NULL)
        OR (subject_type = 'target' AND target_module IS NOT NULL AND target_kind IS NOT NULL)
    );

DROP INDEX IF EXISTS comments_subject_idx;
CREATE INDEX comments_subject_idx
    ON comments (subject_type, target_module, target_kind, subject_id, id DESC);
