-- +goose Up
ALTER TABLE trees
ADD COLUMN provider TEXT,
ADD COLUMN additional_informations JSONB;

ALTER TABLE tree_clusters
ADD COLUMN provider TEXT,
ADD COLUMN additional_informations JSONB;

ALTER TABLE vehicles
ADD COLUMN provider TEXT,
ADD COLUMN additional_informations JSONB;

ALTER TABLE sensors
ADD COLUMN provider TEXT,
ADD COLUMN additional_informations JSONB;

ALTER TABLE flowerbeds
ADD COLUMN provider TEXT,
ADD COLUMN additional_informations JSONB;

ALTER TABLE watering_plans
ADD COLUMN provider TEXT,
ADD COLUMN additional_informations JSONB;


-- +goose Down
ALTER TABLE trees
DROP COLUMN provider,
DROP COLUMN additional_informations;

ALTER TABLE tree_clusters
DROP COLUMN provider,
DROP COLUMN additional_informations;

ALTER TABLE vehicles
DROP COLUMN provider,
DROP COLUMN additional_informations;

ALTER TABLE sensors
DROP COLUMN provider,
DROP COLUMN additional_informations;

ALTER TABLE flowerbeds
DROP COLUMN provider,
DROP COLUMN additional_informations;

ALTER TABLE watering_plans
DROP COLUMN provider,
DROP COLUMN additional_informations;
