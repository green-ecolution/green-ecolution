-- +goose Up
-- +goose StatementBegin
DROP TRIGGER IF EXISTS update_flowerbeds_updated_at ON flowerbeds;
DROP TABLE IF EXISTS flowerbed_images;
DROP TABLE IF EXISTS flowerbeds;
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
CREATE TABLE IF NOT EXISTS flowerbeds (
  id SERIAL PRIMARY KEY,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  sensor_id INT,
  size FLOAT NOT NULL,
  description TEXT NOT NULL,
  number_of_plants INT NOT NULL DEFAULT 0,
  moisture_level FLOAT NOT NULL,
  address TEXT NOT NULL,
  archived BOOLEAN NOT NULL DEFAULT FALSE,
  latitude FLOAT NOT NULL,
  longitude FLOAT NOT NULL,
  geometry GEOMETRY(Polygon, 4326),
  provider TEXT,
  additional_informations JSONB,
  region_id INT
);

ALTER TABLE flowerbeds ADD FOREIGN KEY (region_id) REFERENCES regions(id);

CREATE TRIGGER update_flowerbeds_updated_at
BEFORE UPDATE ON flowerbeds
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE IF NOT EXISTS flowerbed_images (
  flowerbed_id INT,
  image_id INT,
  PRIMARY KEY (flowerbed_id, image_id),
  FOREIGN KEY (flowerbed_id) REFERENCES flowerbeds(id),
  FOREIGN KEY (image_id) REFERENCES images(id)
);
-- +goose StatementEnd
