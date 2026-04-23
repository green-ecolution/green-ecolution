-- +goose Up
-- +goose StatementBegin
INSERT INTO tree_clusters (id, name, watering_status, moisture_level, region_id, address, description, soil_condition, latitude, longitude, geometry)
VALUES 
  (1, 'Flensburger Stadion', 'good', 0.75, 1, 'Am Stadion', 'Alle Bäume am Stadion', 'sandig', 54.820940, 9.489022, ST_SetSRID(ST_MakePoint(54.820940, 9.489022), 4326)),
  (2, 'Sankt-Jürgen-Platz', 'moderate', 0.5, 1, 'Ulmenstraße', 'Bäume beim Sankt-Jürgen-Platz', 'schluffig', 54.78805731048199, 9.44400186680097, ST_SetSRID(ST_MakePoint(54.78805731048199, 9.44400186680097), 4326)),
  (3, 'Solitüde Strand', 'unknown', 0.7, 1, 'Solitüde Strand', 'Alle Bäume am Strand', 'schluffig', 54.802163, 9.446398, ST_SetSRID(ST_MakePoint(54.802163, 9.446398), 4326));   
ALTER SEQUENCE tree_clusters_id_seq RESTART WITH 9;

INSERT INTO sensors (id, status, latitude, longitude, geometry)
VALUES
    ('sensor-1', 'online', 54.82124518093376, 9.485702120628517, ST_SetSRID(ST_MakePoint(54.82124518093376, 9.485702120628517), 4326));

INSERT INTO sensors (id, status, latitude, longitude, geometry)
VALUES
    ('sensor-2', 'offline', 54.78780993841013, 9.444052105200551, ST_SetSRID(ST_MakePoint(54.78780993841013, 9.444052105200551), 4326));

INSERT INTO sensors (id, status, latitude, longitude, geometry)
VALUES
    ('sensor-3', 'unknown', 54.77933725347423, 9.426465409018832, ST_SetSRID(ST_MakePoint(54.77933725347423, 9.426465409018832), 4326));

INSERT INTO sensors (id, status, latitude, longitude, geometry)
VALUES
    ('sensor-4', 'online', 54.82078826498143, 9.489684366114483, ST_SetSRID(ST_MakePoint(54.82078826498143, 9.489684366114483), 4326));


INSERT INTO trees (id, tree_cluster_id, sensor_id, planting_year, species, number, latitude, longitude, geometry, watering_status, description)
VALUES 
  (1, 1, 'sensor-1', 2021, 'Quercus robur', 1005, 54.82124518093376, 9.485702120628517, ST_SetSRID(ST_MakePoint(54.82124518093376, 9.485702120628517), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (2, 1, NULL, 2022, 'Quercus robur', 1006, 54.8215076622281, 9.487153277881877, ST_SetSRID(ST_MakePoint(54.8215076622281, 9.487153277881877), 4326), 'good', ''),
  (3, 1, NULL, 2023, 'Quercus robur', 1007, 54.82078826498143, 9.489684366114483, ST_SetSRID(ST_MakePoint(54.82078826498143, 9.489684366114483), 4326), 'moderate', ''),
  (4, 2, 'sensor-2', 2020, 'Quercus robur', 1008, 54.78780993841013, 9.444052105200551, ST_SetSRID(ST_MakePoint(54.78780993841013, 9.444052105200551), 4326), 'bad', ''),
  (5, 2, NULL, 2021, 'Quercus robur', 1009, 54.78836553796373, 9.444075995492044, ST_SetSRID(ST_MakePoint(54.78836553796373, 9.444075995492044), 4326), 'unknown', ''),
  (6, 2, NULL, 2022, 'Quercus robur', 1010, 54.787768612518455, 9.443996361187065, ST_SetSRID(ST_MakePoint(54.787768612518455, 9.443996361187065), 4326), 'good', '');
-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
DELETE FROM trees;
DELETE FROM tree_clusters;
DELETE FROM sensors;
-- +goose StatementEnd
