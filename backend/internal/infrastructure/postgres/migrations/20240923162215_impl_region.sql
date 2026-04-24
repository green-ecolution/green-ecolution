-- +goose Up
-- +goose StatementBegin
CREATE TABLE regions (
  id SERIAL PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  geometry GEOMETRY(MultiPolygon, 4326)
);
-- +goose StatementEnd

-- +goose StatementBegin
ALTER TABLE tree_clusters DROP COLUMN region;
ALTER TABLE tree_clusters ADD COLUMN region_id INT;
ALTER TABLE tree_clusters ADD FOREIGN KEY (region_id) REFERENCES regions(id);

ALTER TABLE flowerbeds DROP COLUMN region;
ALTER TABLE flowerbeds ADD COLUMN region_id INT;
ALTER TABLE flowerbeds ADD FOREIGN KEY (region_id) REFERENCES regions(id);
-- +goose StatementEnd

CREATE TRIGGER update_region_updated_at
BEFORE UPDATE ON regions
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- +goose Down
-- +goose StatementBegin
ALTER TABLE tree_clusters DROP COLUMN region_id;
ALTER TABLE tree_clusters ADD COLUMN region TEXT;

ALTER TABLE flowerbeds DROP COLUMN region_id;
ALTER TABLE flowerbeds ADD COLUMN region TEXT;

DROP TRIGGER IF EXISTS update_region_updated_at ON regions;
DROP TABLE IF EXISTS regions;
-- +goose StatementEnd
