-- Free-text comments on a tree cluster or a watering plan. The subject is
-- polymorphic, so there is no cascading FK: the parent delete flows in
-- ClusterService and WateringPlanService remove the rows. No created_at column
-- because ids are UUID v7 and carry the timestamp. No organization_id because
-- visibility is derived from the parent, which would otherwise go stale on a
-- responsibility transfer. edited_at is NULL until the author edits the body,
-- so a never-edited comment carries no timestamp.
CREATE TABLE comments (
    id           uuid PRIMARY KEY,
    edited_at    timestamp,
    subject_type text NOT NULL CHECK (subject_type IN ('tree_cluster', 'watering_plan')),
    subject_id   uuid NOT NULL,
    author_id    uuid NOT NULL,
    body         text NOT NULL
);

CREATE INDEX comments_subject_idx ON comments (subject_type, subject_id, id DESC);
