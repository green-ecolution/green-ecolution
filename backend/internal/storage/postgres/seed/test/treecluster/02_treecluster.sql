-- +goose Up
-- +goose StatementBegin
INSERT INTO tree_clusters (id, name, watering_status, moisture_level, region_id, address, description, soil_condition, latitude, longitude, geometry)
VALUES 
  (1, 'Solitüde Strand', 'good', 0.75, 1, 'Solitüde Strand', 'Alle Bäume am Strand', 'sandig', 54.820940, 9.489022, ST_SetSRID(ST_MakePoint(54.820940, 9.489022), 4326)),
  (2, 'Sankt-Jürgen-Platz', 'moderate', 0.5, 1, 'Ulmenstraße', 'Bäume beim Sankt-Jürgen-Platz', 'schluffig', 54.78805731048199, 9.44400186680097, ST_SetSRID(ST_MakePoint(54.78805731048199, 9.44400186680097), 4326)),
  (3, 'Flensburger Stadion', 'unknown', 0.7, 1, 'Flensburger Stadion', 'Alle Bäume in der Gegend des Stadions in Mürwik', 'schluffig', 54.802163, 9.446398, ST_SetSRID(ST_MakePoint(54.802163, 9.446398), 4326)),
  (4, 'Campus Hochschule', 'bad', 0.1, 4, 'Thomas-Finke Straße', 'Gruppe ist besonders anfällig', 'schluffig', 54.776613, 9.454303, ST_SetSRID(ST_MakePoint(54.776613, 9.454303), 4326)),
  (5, 'Mathildenstraße', 'moderate', 0.4, 10, 'Mathildenstraße', 'Sehr enge Straße und dadurch schlecht zu bewässern.', 'schluffig', 54.782402, 9.424270, ST_SetSRID(ST_MakePoint(54.782402, 9.424270), 4326)),
  (6, 'Nordstadt', 'good', 0.6, 13, 'Apenrader Straße', 'Guter Baumbestand mit großen Kronen.', 'sandig', 54.807162, 9.423138, ST_SetSRID(ST_MakePoint(54.807162, 9.423138), 4326)),
  (7, 'TSB Neustadt', 'good', 0.75, 13, 'Ecknerstraße', 'Kleiner Baumbestand.', 'sandig', 54.797162, 9.419620, ST_SetSRID(ST_MakePoint(54.797162, 9.419620), 4326)),
  (8, 'Gewerbegebiet Süd', 'bad', 0.1, NULL, 'Schleswiger Straße', 'Sehr viel versiegelter Boden.', 'sandig', 54.768115, 9.435285, ST_SetSRID(ST_MakePoint(54.768115, 9.435285), 4326));

INSERT INTO tree_clusters (id, name, watering_status, moisture_level, region_id, address, description, soil_condition, latitude, longitude, geometry, provider, additional_informations)
VALUES 
(50, 'Gewerbegebiet Süd', 'bad', 0.1, NULL, 'Schleswiger Straße', 'Sehr viel versiegelter Boden.', 'sandig', 54.768115, 9.435285, ST_SetSRID(ST_MakePoint(54.768115, 9.435285), 4326), 'test-provider', '{"foo":"bar"}');   

ALTER SEQUENCE tree_clusters_id_seq RESTART WITH 9;
-- +goose StatementEnd

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

INSERT INTO sensors (id, status, latitude, longitude, geometry)
VALUES
    ('sensor-11', 'online', 54.82078826498143, 9.489684366114483, ST_SetSRID(ST_MakePoint(54.82078826498143, 9.489684366114483), 4326));
INSERT INTO sensors (id, status, latitude, longitude, geometry)
VALUES
    ('sensor-12', 'online', 54.82078826498143, 9.489684366114483, ST_SetSRID(ST_MakePoint(54.82078826498143, 9.489684366114483), 4326));


INSERT INTO sensor_data (sensor_id, created_at, data)
VALUES 
   ('sensor-11', '2025-01-03 00:41:07.076949', '{"device": "sensor-123", "battery": 34.0, "humidity": 50.0, "temperature": 20.0, "watermarks":[{"centibar": 38, "resistance": 23, "depth": 30}, {"centibar": 38, "resistance": 23, "depth": 60}, {"centibar": 38, "resistance": 23, "depth": 90}]}');

INSERT INTO sensor_data (sensor_id, created_at, data)
VALUES 
   ('sensor-11', '2025-01-03 00:41:08.076949', '{"device": "sensor-123", "battery": 99.0, "humidity": 50.0, "temperature": 20.0, "watermarks":[{"centibar": 38, "resistance": 23, "depth": 30}, {"centibar": 38, "resistance": 23, "depth": 60}, {"centibar": 38, "resistance": 99, "depth": 90}]}');

INSERT INTO sensor_data (sensor_id, created_at, data)
VALUES 
   ('sensor-12', '2025-01-03 00:41:09.076949', '{"device": "sensor-123", "battery": 34.0, "humidity": 50.0, "temperature": 20.0, "watermarks":[{"centibar": 38, "resistance": 23, "depth": 30}, {"centibar": 38, "resistance": 23, "depth": 60}, {"centibar": 38, "resistance": 23, "depth": 90}]}');

INSERT INTO sensor_data (sensor_id, created_at, data)
VALUES 
   ('sensor-12', '2025-01-03 00:41:10.076949', '{"device": "sensor-123", "battery": 99.0, "humidity": 50.0, "temperature": 20.0, "watermarks":[{"centibar": 38, "resistance": 23, "depth": 30}, {"centibar": 38, "resistance": 23, "depth": 60}, {"centibar": 38, "resistance": 99, "depth": 90}]}');

INSERT INTO trees (id, tree_cluster_id, sensor_id, planting_year, species, number, latitude, longitude, geometry, watering_status, description)
VALUES 
  (1, 1, 'sensor-1', 2021, 'Quercus robur', 1005, 54.82124518093376, 9.485702120628517, ST_SetSRID(ST_MakePoint(54.82124518093376, 9.485702120628517), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (2, 1, NULL, 2022, 'Quercus robur', 1006, 54.8215076622281, 9.487153277881877, ST_SetSRID(ST_MakePoint(54.8215076622281, 9.487153277881877), 4326), 'good', ''),
  (3, 1, NULL, 2023, 'Quercus robur', 1007, 54.82078826498143, 9.489684366114483, ST_SetSRID(ST_MakePoint(54.82078826498143, 9.489684366114483), 4326), 'moderate', ''),
  (4, 2, 'sensor-2', 2020, 'Quercus robur', 1008, 54.78780993841013, 9.444052105200551, ST_SetSRID(ST_MakePoint(54.78780993841013, 9.444052105200551), 4326), 'bad', ''),
  (5, 2, NULL, 2021, 'Quercus robur', 1009, 54.78836553796373, 9.444075995492044, ST_SetSRID(ST_MakePoint(54.78836553796373, 9.444075995492044), 4326), 'unknown', ''),
  (6, 2, NULL, 2022, 'Quercus robur', 1010, 54.787768612518455, 9.443996361187065, ST_SetSRID(ST_MakePoint(54.787768612518455, 9.443996361187065), 4326), 'good', ''),
  (7, 5, NULL, 2021, 'Fraxinus ornus Obelisk', 1021, 54.782630, 9.423792, ST_SetSRID(ST_MakePoint(54.782630, 9.423792), 4326), 'good', ''),
  (8, 5, NULL, 2021, 'Fraxinus ornus Obelisk', 1022, 54.782463, 9.423727, ST_SetSRID(ST_MakePoint(54.782463, 9.423727), 4326), 'unknown', ''),
  (9, 5, NULL, 2021, 'Fraxinus ornus Obelisk', 1023, 54.782296, 9.424178, ST_SetSRID(ST_MakePoint(54.782296, 9.424178), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (10, 5, NULL, 2022, 'Fraxinus ornus Obelisk', 1024, 54.782043, 9.424188, ST_SetSRID(ST_MakePoint(54.782043, 9.424188), 4326), 'moderate', ''),
  (11, 5, NULL, 2022, 'Fraxinus ornus Obelisk', 1025, 54.781753, 9.424936, ST_SetSRID(ST_MakePoint(54.781753, 9.424936), 4326), 'moderate', ''),
  (12, 4, NULL, 2022, 'Tilia intermedia', 1026, 54.776145, 9.449785, ST_SetSRID(ST_MakePoint(54.776145, 9.449785), 4326), 'moderate', ''),
  (13, 4, NULL, 2021, 'Tilia intermedia', 1027, 54.776120, 9.450891, ST_SetSRID(ST_MakePoint(54.776120, 9.450891), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (14, 4, NULL, 2021, 'Tilia intermedia', 1028, 54.776058, 9.450311, ST_SetSRID(ST_MakePoint(54.776058, 9.450311), 4326), 'bad', ''),
  (15, 4, NULL, 2021, 'Tilia intermedia', 1029, 54.775709, 9.447762, ST_SetSRID(ST_MakePoint(54.775709, 9.447762), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (16, 3, NULL, 2023, 'Betula pendula', 1034, 54.801718, 9.444797, ST_SetSRID(ST_MakePoint(54.801718, 9.444797), 4326), 'good', ''),
  (17, 3, NULL, 2023, 'Betula pendula', 1035, 54.800797, 9.444271, ST_SetSRID(ST_MakePoint(54.800797, 9.444271), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (18, 3, NULL, 2023, 'Betula pendula', 1036, 54.801539, 9.446741, ST_SetSRID(ST_MakePoint(54.801539, 9.446741), 4326), 'unknown', ''),
  (19, 3, NULL, 2023, 'Betula pendula', 1037, 54.799796, 9.443927, ST_SetSRID(ST_MakePoint(54.799796, 9.443927), 4326), 'unknown', ''),
  (20, 3, NULL, 2023, 'Betula pendula', 1038, 54.804052, 9.447900, ST_SetSRID(ST_MakePoint(54.804052, 9.447900), 4326), 'unknown', ''),
  (21, 6, NULL, 2023, 'Acer platanoides Schwedleri', 1039, 54.806287, 9.423469, ST_SetSRID(ST_MakePoint(54.806287, 9.423469), 4326), 'unknown', ''),
  (22, 6, NULL, 2023, 'Acer platanoides Schwedleri', 1040, 54.807212, 9.422752, ST_SetSRID(ST_MakePoint(54.807212, 9.422752), 4326), 'unknown', ''),
  (23, 6, NULL, 2023, 'Acer platanoides Schwedleri', 1041, 54.806606, 9.422773, ST_SetSRID(ST_MakePoint(54.806606, 9.422773), 4326), 'good', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (24, 6, NULL, 2023, 'Acer platanoides Schwedleri', 1042, 54.807787, 9.422354, ST_SetSRID(ST_MakePoint(54.807787, 9.422354), 4326), 'good', 'Dieser Baum wurde im August das letzte mal gestuzt'),

  (25, 50, NULL, 2023, 'Acer platanoides Schwedleri', 1039, 54.806287, 9.423469, ST_SetSRID(ST_MakePoint(54.806287, 9.423469), 4326), 'unknown', ''),
  (26, 50, 'sensor-11', 2023, 'Acer platanoides Schwedleri', 1040, 54.807212, 9.422752, ST_SetSRID(ST_MakePoint(54.807212, 9.422752), 4326), 'unknown', ''),
  (27, 50, NULL, 2023, 'Acer platanoides Schwedleri', 1041, 54.806606, 9.422773, ST_SetSRID(ST_MakePoint(54.806606, 9.422773), 4326), 'good', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (28, 50, 'sensor-12', 2023, 'Acer platanoides Schwedleri', 1042, 54.807787, 9.422354, ST_SetSRID(ST_MakePoint(54.807787, 9.422354), 4326), 'good', 'Dieser Baum wurde im August das letzte mal gestuzt');

-- +goose Down
DELETE FROM trees;
DELETE FROM tree_clusters;
DELETE FROM sensor_data;
