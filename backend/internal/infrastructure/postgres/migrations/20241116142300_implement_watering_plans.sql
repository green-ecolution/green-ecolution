-- +goose Up
CREATE TYPE watering_plan_status AS ENUM ('planned', 'active', 'canceled', 'finished', 'not competed', 'unknown');

CREATE TABLE IF NOT EXISTS watering_plans (
  id SERIAL PRIMARY KEY,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  date DATE NOT NULL,
  description TEXT NOT NULL,
  watering_plan_status watering_plan_status NOT NULL DEFAULT 'unknown',
  distance FLOAT,
  total_water_required FLOAT
);

CREATE TABLE IF NOT EXISTS user_watering_plans (
  user_id UUID NOT NULL,
  watering_plan_id INT NOT NULL,
  PRIMARY KEY (user_id, watering_plan_id),
  FOREIGN KEY (watering_plan_id) REFERENCES watering_plans(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS vehicle_watering_plans (
  vehicle_id INT NOT NULL,
  watering_plan_id INT NOT NULL,
  PRIMARY KEY (vehicle_id, watering_plan_id),
  FOREIGN KEY (vehicle_id) REFERENCES vehicles(id) ON DELETE CASCADE,
  FOREIGN KEY (watering_plan_id) REFERENCES watering_plans(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS tree_cluster_watering_plans (
  tree_cluster_id INT NOT NULL,
  watering_plan_id INT NOT NULL,
  PRIMARY KEY (tree_cluster_id, watering_plan_id),
  FOREIGN KEY (tree_cluster_id) REFERENCES tree_clusters(id) ON DELETE CASCADE,
  FOREIGN KEY (watering_plan_id) REFERENCES watering_plans(id) ON DELETE CASCADE
);

CREATE TRIGGER update_watering_plans_updated_at
BEFORE UPDATE ON watering_plans
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- +goose Down
DROP TRIGGER IF EXISTS update_watering_plans_updated_at ON watering_plans;
DROP TABLE IF EXISTS user_watering_plans;
DROP TABLE IF EXISTS vehicle_watering_plans;
DROP TABLE IF EXISTS tree_cluster_watering_plans;
DROP TABLE IF EXISTS watering_plans;

DROP TYPE IF EXISTS watering_plan_status;
