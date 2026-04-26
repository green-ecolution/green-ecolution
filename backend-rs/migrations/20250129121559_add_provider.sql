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


