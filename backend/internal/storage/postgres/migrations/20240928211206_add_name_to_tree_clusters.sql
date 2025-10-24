-- +goose Up
ALTER TABLE tree_clusters
ADD COLUMN name TEXT NOT NULL;

-- +goose Down
ALTER TABLE tree_clusters
DROP COLUMN name;