CREATE EXTENSION IF NOT EXISTS postgis;

CREATE TABLE IF NOT EXISTS images (
  id SERIAL PRIMARY KEY,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  url TEXT NOT NULL,
  filename TEXT,
  mime_type TEXT
);

CREATE TABLE IF NOT EXISTS vehicles (
  id SERIAL PRIMARY KEY,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  number_plate TEXT NOT NULL,
  description TEXT NOT NULL,
  water_capacity FLOAT NOT NULL
);

CREATE TABLE IF NOT EXISTS user_vehicles (
  user_id UUID NOT NULL,
  vehicle_id INT NOT NULL,
  PRIMARY KEY (user_id, vehicle_id),
  FOREIGN KEY (vehicle_id) REFERENCES vehicles(id)
);

CREATE TYPE watering_status AS ENUM ('good', 'moderate', 'bad', 'unknown');
CREATE TYPE tree_soil_condition AS ENUM ('schluffig', 'sandig', 'lehmig', 'tonig', 'unknown');

CREATE TABLE IF NOT EXISTS tree_clusters (
  id SERIAL PRIMARY KEY,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  watering_status watering_status NOT NULL DEFAULT 'unknown',
  last_watered TIMESTAMP,
  -- last_watered_by_vehicle INT,
  moisture_level FLOAT NOT NULL,
  region TEXT NOT NULL,
  address TEXT NOT NULL,
  description TEXT NOT NULL,
  archived BOOLEAN NOT NULL DEFAULT FALSE,
  soil_condition tree_soil_condition NOT NULL DEFAULT 'unknown',
  latitude FLOAT,
  longitude FLOAT,
  geometry GEOMETRY(Point, 4326)
);

CREATE TYPE sensor_status AS ENUM ('online', 'offline', 'unknown');

CREATE TABLE IF NOT EXISTS sensors (
  id SERIAL PRIMARY KEY,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  status sensor_status NOT NULL DEFAULT 'unknown'
);

CREATE TABLE IF NOT EXISTS sensor_data (
  id SERIAL PRIMARY KEY,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  data JSONB NOT NULL,
  sensor_id INT NOT NULL,
  FOREIGN KEY (sensor_id) REFERENCES sensors(id)
);

CREATE TABLE IF NOT EXISTS trees (
  id SERIAL PRIMARY KEY,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  tree_cluster_id INT,
  sensor_id INT,
  age INT NOT NULL,
  height_above_sea_level FLOAT NOT NULL,
  planting_year INT NOT NULL,
  species TEXT NOT NULL,
  tree_number TEXT NOT NULL,
  latitude FLOAT NOT NULL,
  longitude FLOAT NOT NULL,
  watering_status watering_status NOT NULL DEFAULT 'unknown',
  geometry GEOMETRY(Point, 4326),
  FOREIGN KEY (sensor_id) REFERENCES sensors(id),
  FOREIGN KEY (tree_cluster_id) REFERENCES tree_clusters(id)
);

CREATE TABLE IF NOT EXISTS tree_images (
  tree_id INT,
  image_id INT,
  PRIMARY KEY (tree_id, image_id),
  FOREIGN KEY (tree_id) REFERENCES trees(id),
  FOREIGN KEY (image_id) REFERENCES images(id)
);

CREATE TABLE IF NOT EXISTS flowerbeds (
  id SERIAL PRIMARY KEY,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  sensor_id INT,
  size FLOAT NOT NULL,
  description TEXT NOT NULL,
  number_of_plants INT NOT NULL DEFAULT 0,
  moisture_level FLOAT NOT NULL,
  region TEXT NOT NULL,
  address TEXT NOT NULL,
  archived BOOLEAN NOT NULL DEFAULT FALSE,
  latitude FLOAT NOT NULL,
  longitude FLOAT NOT NULL,
  geometry GEOMETRY(Polygon, 4326)
);

CREATE TABLE IF NOT EXISTS flowerbed_images (
  flowerbed_id INT,
  image_id INT,
  PRIMARY KEY (flowerbed_id, image_id),
  FOREIGN KEY (flowerbed_id) REFERENCES flowerbeds(id),
  FOREIGN KEY (image_id) REFERENCES images(id)
);

CREATE OR REPLACE FUNCTION update_updated_at_column()
  RETURNS TRIGGER
  AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$
language 'plpgsql';

CREATE TRIGGER update_images_updated_at
BEFORE UPDATE ON images
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_vehicles_updated_at
BEFORE UPDATE ON vehicles
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_tree_clusters_updated_at
BEFORE UPDATE ON tree_clusters
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_sensors_updated_at
BEFORE UPDATE ON sensors
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_sensor_data_updated_at
BEFORE UPDATE ON sensor_data
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_trees_updated_at
BEFORE UPDATE ON trees
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_flowerbeds_updated_at
BEFORE UPDATE ON flowerbeds
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

