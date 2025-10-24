-- +goose Up
-- +goose StatementBegin
INSERT INTO vehicles (id, number_plate, description, water_capacity, type, status, driving_license, model, width, height, length, weight) 
VALUES 
  (1, 'B-1111', 'Test vehicle 1', 1000.0, 'trailer', 'active', 'BE', 'LK1615/17 - Conrad - MAN TGE 3.180', 2.0, 1.5, 2.0, 3.5),
  (2, 'B-2222', 'Test vehicle 2', 1000.0, 'transporter', 'unknown', 'C', 'Actros L Mercedes Benz', 2.4, 2.1, 5.0, 2.5),
  (3, 'B-3333', 'Test vehicle 3', 2800.0, 'transporter', 'available', 'C', 'Ford Ranger XL', 2.55, 4.0, 9.09, 26),
  (4, 'B-4444', 'Test vehicle 4', 1000.0, 'trailer', 'available', 'BE', 'VW Crafter Pritsche', 2.4, 2.1, 5.0, 5.5);
ALTER SEQUENCE vehicles_id_seq RESTART WITH 5;

INSERT INTO tree_clusters (id, name, watering_status, moisture_level, region_id, address, description, soil_condition, latitude, longitude, geometry)
VALUES 
  (1, 'Solitüde Strand', 'good', 0.75, 1, 'Solitüde Strand', 'Alle Bäume am Strand', 'sandig', 54.82128536520703, 9.488152515892045, ST_SetSRID(ST_MakePoint(54.82128536520703, 9.488152515892045), 4326)),
  (2, 'Sankt-Jürgen-Platz', 'moderate', 0.5, 1, 'Ulmenstraße', 'Bäume beim Sankt-Jürgen-Platz', 'schluffig', 54.78805731048199, 9.44400186680097, ST_SetSRID(ST_MakePoint(54.78805731048199, 9.44400186680097), 4326)),
  (3, 'Flensburger Stadion', 'unknown', 0.7, 1, 'Flensburger Stadion', 'Alle Bäume in der Gegend des Stadions in Mürwik', 'schluffig', 54.802163, 9.446398, ST_SetSRID(ST_MakePoint(54.802163, 9.446398), 4326)),
  (4, 'Campus Hochschule', 'bad', 0.1, 4, 'Thomas-Finke Straße', 'Gruppe ist besonders anfällig', 'schluffig', 54.77576059694547, 9.450720736264868, ST_SetSRID(ST_MakePoint(54.77576059694547, 9.450720736264868), 4326)),
  (5, 'Mathildenstraße', 'bad', 0.4, 10, 'Mathildenstraße', 'Sehr enge Straße und dadurch schlecht zu bewässern.', 'schluffig', 54.78219253876479, 9.423978982828825, ST_SetSRID(ST_MakePoint(54.78219253876479, 9.423978982828825), 4326)),
  (6, 'Nordstadt', 'unknown', 0.6, 13, 'Apenrader Straße', 'Guter Baumbestand mit großen Kronen.', 'sandig', 54.807162, 9.423138, ST_SetSRID(ST_MakePoint(54.807162, 9.423138), 4326)),
  (7, 'TSB Neustadt', 'good', 0.75, 13, 'Ecknerstraße', 'Kleiner Baumbestand.', 'sandig', 54.797162, 9.419620, ST_SetSRID(ST_MakePoint(54.797162, 9.419620), 4326)),
  (8, 'Seniorenanlage Valentinerhof', 'bad', 0.1, 13, 'Auf dem Geländer der Seniorenanlage', 'Sehr viel versiegelter Boden.', 'sandig', 54.76994251235151, 9.441111747447234, ST_SetSRID(ST_MakePoint(54.76994251235151, 9.441111747447234), 4326)),
  (9, 'Peelwatt', 'unknown', 0.1, 13, 'Peelwatt halt', 'Sehr viel versiegelter Boden.', 'sandig', 54.76671656688957, 9.456136954289867, ST_SetSRID(ST_MakePoint(54.76671656688957, 9.456136954289867), 4326)),
  (10, 'Lautrupsbach', 'moderate', 0.1, 13, 'An der Nordstraße', 'Sehr viel versiegelter Boden.', 'sandig', 54.79265065021804, 9.454269041383837, ST_SetSRID(ST_MakePoint(54.76671656688957, 9.454269041383837), 4326));
ALTER SEQUENCE tree_clusters_id_seq RESTART WITH 11;

INSERT INTO sensors (id, status, latitude, longitude, geometry)
VALUES
    ('sensor-1', 'offline', 54.82124518093376, 9.485702120628517, ST_SetSRID(ST_MakePoint(54.82124518093376, 9.485702120628517), 4326)),
    ('sensor-2', 'offline', 54.8215076622281, 9.487153277881877, ST_SetSRID(ST_MakePoint(54.8215076622281, 9.487153277881877), 4326)),
    ('sensor-3', 'offline', 54.78780993841013, 9.444052105200551, ST_SetSRID(ST_MakePoint(54.78780993841013, 9.444052105200551), 4326)),
    ('sensor-4', 'offline', 54.775679885633636, 9.451171073968197, ST_SetSRID(ST_MakePoint(54.775679885633636, 9.451171073968197), 4326)),
    ('sensor-5', 'offline', 54.782630, 9.423792, ST_SetSRID(ST_MakePoint(54.782630, 9.423792), 4326)),
    ('sensor-6', 'offline', 54.796916, 9.421332, ST_SetSRID(ST_MakePoint(54.796916, 9.421332), 4326)),
    ('sensor-7', 'offline', 54.7697451282801, 9.439562555553788, ST_SetSRID(ST_MakePoint(54.7697451282801, 9.439562555553788), 4326)),
    ('sensor-8', 'offline', 54.792472, 9.450000, ST_SetSRID(ST_MakePoint(54.792472, 9.450000), 4326));

INSERT INTO trees (tree_cluster_id, sensor_id, planting_year, species, number, latitude, longitude, geometry, watering_status, description)
VALUES
  (1, 'sensor-1', 2023, 'Quercus robur', 1005, 54.82124518093376, 9.485702120628517, ST_SetSRID(ST_MakePoint(54.82124518093376, 9.485702120628517), 4326), 'good', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (1, 'sensor-2', 2023, 'Quercus robur', 1006, 54.8215076622281, 9.487153277881877, ST_SetSRID(ST_MakePoint(54.8215076622281, 9.487153277881877), 4326), 'good', ''),
  (1, NULL, 2023, 'Quercus robur', 1007, 54.82078826498143, 9.489684366114483, ST_SetSRID(ST_MakePoint(54.82078826498143, 9.489684366114483), 4326), 'unknown', ''),
  (1, NULL, 2023, 'Quercus robur', 1001, 54.820834078576304, 9.486398528109389, ST_SetSRID(ST_MakePoint(54.820834078576304, 9.486398528109389), 4326), 'unknown', ''),
  (1, NULL, 2023, 'Quercus robur', 1002, 54.82008971976509, 9.488979617332221, ST_SetSRID(ST_MakePoint(54.82008971976509, 9.488979617332221), 4326), 'unknown', ''),
  (1, NULL, 2023, 'Quercus robur', 1003, 54.82061210171266, 9.486168703385617, ST_SetSRID(ST_MakePoint(54.82061210171266, 9.486168703385617), 4326), 'unknown', ''),
  (1, NULL, 2023, 'Quercus robur', 1004, 54.8199067243877, 9.487106513347264, ST_SetSRID(ST_MakePoint(54.8199067243877, 9.487106513347264), 4326), 'unknown', ''),
  (1, NULL, 2023, 'Quercus robur', 2001, 54.821248829192285, 9.48996664076417, ST_SetSRID(ST_MakePoint(54.821248829192285, 9.48996664076417), 4326), 'unknown', ''),

  (2, 'sensor-3', 2022, 'Quercus robur', 1008, 54.78780993841013, 9.444052105200551, ST_SetSRID(ST_MakePoint(54.78780993841013, 9.444052105200551), 4326), 'moderate', ''),
  (2, NULL, 2022, 'Quercus robur', 1009, 54.78836553796373, 9.444075995492044, ST_SetSRID(ST_MakePoint(54.78836553796373, 9.444075995492044), 4326), 'unknown', ''),
  (2, NULL, 2022, 'Quercus robur', 1010, 54.787768612518455, 9.443996361187065, ST_SetSRID(ST_MakePoint(54.787768612518455, 9.443996361187065), 4326), 'unknown', ''),
  (2, NULL, 2022, 'Quercus robur', 1010, 54.78826721846835, 9.443595915277797, ST_SetSRID(ST_MakePoint(54.78826721846835, 9.443595915277797), 4326), 'unknown', ''),
  (2, NULL, 2022, 'Quercus robur', 1010, 54.78810634901004, 9.44443262510434, ST_SetSRID(ST_MakePoint(54.78810634901004, 9.44443262510434), 4326), 'unknown', ''),
  (2, NULL, 2022, 'Quercus robur', 1010, 54.78815894101875, 9.443955271421238, ST_SetSRID(ST_MakePoint(54.78815894101875, 9.443955271421238), 4326), 'unknown', ''),

  (3, NULL, 2023, 'Betula pendula', 1034, 54.801718, 9.444797, ST_SetSRID(ST_MakePoint(54.801718, 9.444797), 4326), 'unknown', ''),
  (3, NULL, 2023, 'Betula pendula', 1035, 54.800797, 9.444271, ST_SetSRID(ST_MakePoint(54.800797, 9.444271), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (3, NULL, 2023, 'Betula pendula', 1036, 54.801539, 9.446741, ST_SetSRID(ST_MakePoint(54.801539, 9.446741), 4326), 'unknown', ''),
  (3, NULL, 2023, 'Betula pendula', 1037, 54.799796, 9.443927, ST_SetSRID(ST_MakePoint(54.799796, 9.443927), 4326), 'unknown', ''),
  (3, NULL, 2023, 'Betula pendula', 1038, 54.804052, 9.447900, ST_SetSRID(ST_MakePoint(54.804052, 9.447900), 4326), 'unknown', ''),

  (4, 'sensor-4', 2022, 'Tilia intermedia', 1029, 54.775679885633636, 9.451171073968197, ST_SetSRID(ST_MakePoint(54.775679885633636, 9.451171073968197), 4326), 'bad', ''),
  (4, NULL, 2022, 'Tilia intermedia', 1027, 54.776120, 9.450891, ST_SetSRID(ST_MakePoint(54.776120, 9.450891), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (4, NULL, 2022, 'Tilia intermedia', 1028, 54.776058, 9.450311, ST_SetSRID(ST_MakePoint(54.776058, 9.450311), 4326), 'unknown', ''),
  (4, NULL, 2022, 'Tilia intermedia', 1029, 54.775709, 9.447762, ST_SetSRID(ST_MakePoint(54.775709, 9.447762), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (4, NULL, 2022, 'Tilia intermedia', 1026, 54.776145, 9.449785, ST_SetSRID(ST_MakePoint(54.776145, 9.449785), 4326), 'unknown', ''),
  (4, NULL, 2022, 'Tilia intermedia', 1026, 54.774986825456224, 9.451846963834953, ST_SetSRID(ST_MakePoint(54.774986825456224, 9.451846963834953), 4326), 'unknown', 'UNSER TEST BAUM'),

  (5, 'sensor-5', 2021, 'Fraxinus ornus Obelisk', 1021, 54.782630, 9.423792, ST_SetSRID(ST_MakePoint(54.782630, 9.423792), 4326), 'bad', ''),
  (5, NULL, 2021, 'Fraxinus ornus Obelisk', 1022, 54.782463, 9.423727, ST_SetSRID(ST_MakePoint(54.782463, 9.423727), 4326), 'unknown', ''),
  (5, NULL, 2021, 'Fraxinus ornus Obelisk', 1023, 54.782296, 9.424178, ST_SetSRID(ST_MakePoint(54.782296, 9.424178), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (5, NULL, 2022, 'Fraxinus ornus Obelisk', 1024, 54.782043, 9.424188, ST_SetSRID(ST_MakePoint(54.782043, 9.424188), 4326), 'unknown', ''),
  (5, NULL, 2022, 'Fraxinus ornus Obelisk', 1025, 54.781753, 9.424936, ST_SetSRID(ST_MakePoint(54.781753, 9.424936), 4326), 'unknown', ''),

  (6, NULL, 2023, 'Acer platanoides Schwedleri', 1039, 54.806287, 9.423469, ST_SetSRID(ST_MakePoint(54.806287, 9.423469), 4326), 'unknown', ''),
  (6, NULL, 2023, 'Acer platanoides Schwedleri', 1040, 54.807212, 9.422752, ST_SetSRID(ST_MakePoint(54.807212, 9.422752), 4326), 'unknown', ''),
  (6, NULL, 2023, 'Acer platanoides Schwedleri', 1041, 54.806606, 9.422773, ST_SetSRID(ST_MakePoint(54.806606, 9.422773), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (6, NULL, 2023, 'Acer platanoides Schwedleri', 1042, 54.807787, 9.422354, ST_SetSRID(ST_MakePoint(54.807787, 9.422354), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),

  (7, 'sensor-6', 2022, 'Acer platanoides Schwedleri', 1043, 54.796916, 9.421332, ST_SetSRID(ST_MakePoint(54.796916, 9.421332), 4326), 'good', ''),
  (7, NULL, 2022, 'Acer platanoides Schwedleri', 1044, 54.797330, 9.419002, ST_SetSRID(ST_MakePoint(54.797330, 9.419002), 4326), 'unknown', ''),
  (7, NULL, 2022, 'Acer platanoides Schwedleri', 1045, 54.797114, 9.417843, ST_SetSRID(ST_MakePoint(54.797114, 9.417843), 4326), 'unknown', ''),

  (8, 'sensor-7', 2023, 'Populus cf. suaveolens', 1052, 54.7697451282801, 9.439562555553788, ST_SetSRID(ST_MakePoint(54.7697451282801, 9.439562555553788), 4326), 'bad', ''),
  (8, NULL, 2023, 'Populus cf. suaveolens', 5555, 54.76932352301634, 9.441299419876234, ST_SetSRID(ST_MakePoint(54.76932352301634, 9.441299419876234), 4326), 'unknown', ''),
  (8, NULL, 2023, 'Populus cf. suaveolens', 4444, 54.76915329290317, 9.441851862902759, ST_SetSRID(ST_MakePoint(54.76915329290317, 9.441851862902759), 4326), 'unknown', ''),
  (8, NULL, 2023, 'Populus cf. suaveolens', 3333, 54.770304653528044, 9.44233994363491, ST_SetSRID(ST_MakePoint(54.770304653528044, 9.44233994363491), 4326), 'unknown', ''),
  (8, NULL, 2023, 'Populus cf. suaveolens', 2222, 54.771043653535294, 9.440740347234932, ST_SetSRID(ST_MakePoint(54.771043653535294, 9.440740347234932), 4326), 'unknown', ''),

  (9, NULL, 2024, 'Tilia x vulgaris', 1015, 54.76752937879732, 9.457372632491829, ST_SetSRID(ST_MakePoint(54.76752937879732, 9.457372632491829), 4326), 'unknown', ''),
  (9, NULL, 2024, 'Tilia x vulgaris', 1015, 54.767564688002714, 9.453443844886783, ST_SetSRID(ST_MakePoint(54.767564688002714, 9.453443844886783), 4326), 'unknown', ''),
  (9, NULL, 2024, 'Tilia x vulgaris', 1015, 54.765620842535895, 9.4575523046762, ST_SetSRID(ST_MakePoint(54.765620842535895, 9.4575523046762), 4326), 'unknown', ''),
  (9, NULL, 2024, 'Tilia x vulgaris', 1015, 54.76725516472003, 9.456833592389275, ST_SetSRID(ST_MakePoint(54.76725516472003, 9.456833592389275), 4326), 'unknown', ''),

  (10, 'sensor-8', 2023, 'Alnus glutinosa', 1030, 54.792472, 9.452773, ST_SetSRID(ST_MakePoint(54.792472, 9.452773), 4326), 'moderate', ''),
  (10, NULL, 2023, 'Alnus glutinosa', 1031, 54.792782, 9.453795, ST_SetSRID(ST_MakePoint(54.792782, 9.453795), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (10, NULL, 2023, 'Alnus glutinosa', 1032, 54.792837, 9.454880, ST_SetSRID(ST_MakePoint(54.792837, 9.454880), 4326), 'unknown', ''),
  (10, NULL, 2023, 'Alnus glutinosa', 1033, 54.792435, 9.455545, ST_SetSRID(ST_MakePoint(54.792435, 9.455545), 4326), 'unknown', ''),

  (NULL, NULL, 2024, 'Carpinus betulus', 1015, 54.783739, 9.426823, ST_SetSRID(ST_MakePoint(54.783739, 9.426823), 4326), 'unknown', ''),
  (NULL, NULL, 2022, 'Carpinus betulus', 1017, 54.785981, 9.430668, ST_SetSRID(ST_MakePoint(54.785981, 9.430668), 4326), 'unknown', ''),
  (NULL, NULL, 2022, 'Carpinus betulus', 1018, 54.786269, 9.431758, ST_SetSRID(ST_MakePoint(54.786269, 9.431758), 4326), 'unknown', ''),
  (NULL, NULL, 2022, 'Carpinus betulus', 1019, 54.787339, 9.431701, ST_SetSRID(ST_MakePoint(54.787339, 9.431701), 4326), 'unknown', ''),
  (NULL, NULL, 2021, 'Carpinus betulus', 1020, 54.786656, 9.432243, ST_SetSRID(ST_MakePoint(54.786656, 9.432243), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (NULL, NULL, 2022, 'Populus cf. suaveolens', 1051, 54.769030, 9.429936, ST_SetSRID(ST_MakePoint(54.769030, 9.429936), 4326), 'unknown', ''),
  (NULL, NULL, 2021, 'Populus cf. suaveolens', 1053, 54.775237, 9.441981, ST_SetSRID(ST_MakePoint(54.775237, 9.441981), 4326), 'unknown', ''),
  (NULL, NULL, 2022, 'Populus cf. suaveolens', 1054, 54.780192, 9.459607, ST_SetSRID(ST_MakePoint(54.780192, 9.459607), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (NULL, NULL, 2021, 'Populus cf. suaveolens', 1055, 54.785043, 9.418210, ST_SetSRID(ST_MakePoint(54.785043, 9.418210), 4326), 'unknown', ''),
  (NULL, NULL, 2022, 'Fraxinus excelsior', 1056, 54.779697, 9.440026, ST_SetSRID(ST_MakePoint(54.779697, 9.440026), 4326), 'unknown', ''),
  (NULL, NULL, 2020, 'Fraxinus excelsior', 1057, 54.785147, 9.438903, ST_SetSRID(ST_MakePoint(54.785147, 9.438903), 4326), 'unknown', ''),
  (NULL, NULL, 2020, 'Fraxinus excelsior', 1058, 54.788205, 9.454699, ST_SetSRID(ST_MakePoint(54.788205, 9.454699), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (NULL, NULL, 2020, 'Fraxinus excelsior', 1059, 54.804054, 9.469544, ST_SetSRID(ST_MakePoint(54.804054, 9.469544), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (NULL, NULL, 2024, 'Acer pseudoplatanus', 1060, 54.813655, 9.477633, ST_SetSRID(ST_MakePoint(54.813655, 9.477633), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (NULL, NULL, 2024, 'Acer pseudoplatanus', 1061, 54.811001, 9.484132, ST_SetSRID(ST_MakePoint(54.811001, 9.484132), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (NULL, NULL, 2024, 'Acer pseudoplatanus', 1062, 54.790366, 9.472744, ST_SetSRID(ST_MakePoint(54.790366, 9.472744), 4326), 'unknown', '');

INSERT INTO trees (tree_cluster_id, sensor_id, planting_year, species, number, latitude, longitude, geometry, watering_status, description, last_watered)
VALUES
  (NULL, NULL, 2023, 'Quercus robur', 1065, 54.780394213230196, 9.417514801025392, ST_SetSRID(ST_MakePoint(54.780394213230196, 9.417514801025392), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt', '2025-02-14 12:34:56');

ALTER SEQUENCE trees_id_seq RESTART WITH 68;

INSERT INTO sensor_data (sensor_id, data)
VALUES
    ('sensor-1', '{
        "device": "sensor-1",
        "temperature": 2.0,
        "humidity": 0.5,
        "battery": 3.943,
        "watermarks": [
            {"resistance": 1022, "centibar": 10, "depth": 30},
            {"resistance": 1110, "centibar": 11, "depth": 60},
            {"resistance": 944, "centibar": 8, "depth": 90}
        ]
    }'),
    ('sensor-1', '{
        "device": "sensor-1",
        "temperature": 2.0,
        "humidity": 0.5,
        "battery": 2.1,
        "watermarks": [
            {"resistance": 800, "centibar": 8, "depth": 30},
            {"resistance": 1000, "centibar": 7, "depth": 60},
            {"resistance": 400, "centibar": 2, "depth": 90}
        ]
    }'),
    ('sensor-2', '{
        "device": "sensor-2",
        "temperature": 2.0,
        "humidity": 0.5,
        "battery": 3.943,
        "watermarks": [
            {"resistance": 1020, "centibar": 90, "depth": 30},
            {"resistance": 1000, "centibar": 80, "depth": 60},
            {"resistance": 900, "centibar": 20, "depth": 90}
        ]
    }'),
    ('sensor-2', '{
        "device": "sensor-2",
        "temperature": 21.0,
        "humidity": 10.0,
        "battery": 3.0,
        "watermarks": [
            {"resistance": 1400, "centibar": 80, "depth": 30},
            {"resistance": 400, "centibar": 40, "depth": 60},
            {"resistance": 200, "centibar": 10, "depth": 90}
        ]
    }'),
    ('sensor-2', '{
        "device": "sensor-2",
        "temperature": 23.0,
        "humidity": 0.5,
        "battery": 2.5,
        "watermarks": [
            {"resistance": 300, "centibar": 20, "depth": 30},
            {"resistance": 500, "centibar": 40, "depth": 60},
            {"resistance": 600, "centibar": 50, "depth": 90}
        ]
    }'),
    ('sensor-2', '{
        "device": "sensor-2",
        "temperature": 10.0,
        "humidity": 4.0,
        "battery": 2.1,
        "watermarks": [
            {"resistance": 1200, "centibar": 12, "depth": 30},
            {"resistance": 1100, "centibar": 11, "depth": 60},
            {"resistance": 1000, "centibar": 10, "depth": 90}
        ]
    }'),
    ('sensor-3', '{
        "device": "sensor-3",
        "temperature": 4.0,
        "humidity": 1,
        "battery": 3.4,
        "watermarks": [
            {"resistance": 800, "centibar": 80, "depth": 30},
            {"resistance": 1000, "centibar": 10, "depth": 60},
            {"resistance": 1200, "centibar": 12, "depth": 90}
        ]
    }'),
    ('sensor-3', '{
        "device": "sensor-3",
        "temperature": 4.0,
        "humidity": 1,
        "battery": 3.4,
        "watermarks": [
            {"resistance": 200, "centibar": 20, "depth": 30},
            {"resistance": 100, "centibar": 10, "depth": 60},
            {"resistance": 230, "centibar": 20, "depth": 90}
        ]
    }'),
    ('sensor-4', '{
        "device": "sensor-4",
        "temperature": 2.0,
        "humidity": 0.5,
        "battery": 3.0,
        "watermarks": [
            {"resistance": 2000, "centibar": 80, "depth": 30},
            {"resistance": 2200, "centibar": 85, "depth": 60},
            {"resistance": 2500, "centibar": 90, "depth": 90}
        ]
    }'),
    ('sensor-4', '{
        "device": "sensor-4",
        "temperature": 2.0,
        "humidity": 10.0,
        "battery": 3.1,
        "watermarks": [
            {"resistance": 2100, "centibar": 10, "depth": 30},
            {"resistance": 2400, "centibar": 20, "depth": 60},
            {"resistance": 2600, "centibar": 30, "depth": 90}
        ]
    }'),
    ('sensor-4', '{
        "device": "sensor-4",
        "temperature": 3.0,
        "humidity": 2.0,
        "battery": 2.8,
        "watermarks": [
            {"resistance": 300, "centibar": 30, "depth": 30},
            {"resistance": 200, "centibar": 20, "depth": 60},
            {"resistance": 100, "centibar": 10, "depth": 90}
        ]
    }'),
    ('sensor-5', '{
        "device": "sensor-5",
        "temperature": 2.23,
        "humidity": 1.5,
        "battery": 3.33,
        "watermarks": [
            {"resistance": 2230, "centibar": 80, "depth": 30},
            {"resistance": 2240, "centibar": 85, "depth": 60},
            {"resistance": 2500, "centibar": 90, "depth": 90}
        ]
    }'),
    ('sensor-5', '{
        "device": "sensor-5",
        "temperature": 2.23,
        "humidity": 1.5,
        "battery": 3.1,
        "watermarks": [
            {"resistance": 3030, "centibar": 70, "depth": 30},
            {"resistance": 2020, "centibar": 60, "depth": 60},
            {"resistance": 1010, "centibar": 50, "depth": 90}
        ]
    }'),
    ('sensor-6', '{
        "device": "sensor-6",
        "temperature": 2.0,
        "humidity": 0.5,
        "battery": 3.7,
        "watermarks": [
            {"resistance": 400, "centibar": 35, "depth": 30},
            {"resistance": 500, "centibar": 40, "depth": 60},
            {"resistance": 600, "centibar": 45, "depth": 90}
        ]
    }'),
    ('sensor-7', '{
        "device": "sensor-8",
        "temperature": 2.23,
        "humidity": 1.5,
        "battery": 3.33,
        "watermarks": [
            {"resistance": 2230, "centibar": 80, "depth": 30},
            {"resistance": 2240, "centibar": 85, "depth": 60},
            {"resistance": 2500, "centibar": 90, "depth": 90}
        ]
    }'),
    ('sensor-7', '{
        "device": "sensor-8",
        "temperature": 10.2,
        "humidity": 1.5,
        "battery": 2.0,
        "watermarks": [
            {"resistance": 1230, "centibar": 50, "depth": 30},
            {"resistance": 1240, "centibar": 55, "depth": 60},
            {"resistance": 1500, "centibar": 60, "depth": 90}
        ]
    }'),
    ('sensor-8', '{
        "device": "sensor-9",
        "temperature": 2.0,
        "humidity": 0.5,
        "battery": 3.7,
        "watermarks": [
            {"resistance": 850, "centibar": 20, "depth": 30},
            {"resistance": 600, "centibar": 30, "depth": 60},
            {"resistance": 100, "centibar": 5, "depth": 90}
        ]
    }'),
    ('sensor-8', '{
        "device": "sensor-9",
        "temperature": 2.0,
        "humidity": 10.0,
        "battery": 3.0,
        "watermarks": [
            {"resistance": 800, "centibar": 50, "depth": 30},
            {"resistance": 900, "centibar": 52, "depth": 60},
            {"resistance": 1000, "centibar": 55, "depth": 90}
        ]
    }');


INSERT INTO watering_plans (id, date, description, status, distance, total_water_required, cancellation_note)
VALUES 
  (1, '2025-09-22', 'New watering plan for the west side of the city', 'planned', 63.0, 720.0, ''),
  (2, '2025-08-03', 'New watering plan for the east side of the city', 'active', 63.0, 600.0, ''),
  (3, '2025-06-12', 'Very important watering plan due to no rainfall', 'finished', 63.0, 1320.0, ''),
  (4, '2025-06-10', 'New watering plan for the south side of the city', 'not competed', 63.0, 600.0, ''),
  (5, '2025-06-04', 'Canceled due to flood', 'canceled', 63.0, 600.0, 'The watering plan was cancelled due to various reasons.');
ALTER SEQUENCE watering_plans_id_seq RESTART WITH 6;

INSERT INTO vehicle_watering_plans (vehicle_id, watering_plan_id) 
VALUES 
  (1, 1),
  (2, 1),
  (2, 2),
  (2, 3),
  (2, 4),
  (2, 5);

INSERT INTO user_watering_plans (user_id, watering_plan_id) 
VALUES 
  ('6a1078e8-80fd-458f-b74e-e388fe2dd6ab', 1),
  ('6a1078e8-80fd-458f-b74e-e388fe2dd6ab', 2),
  ('6a1078e8-80fd-458f-b74e-e388fe2dd6ab', 3),
  ('6a1078e8-80fd-458f-b74e-e388fe2dd6ab', 4),
  ('6a1078e8-80fd-458f-b74e-e388fe2dd6ab', 5);

INSERT INTO tree_cluster_watering_plans (tree_cluster_id, watering_plan_id, consumed_water)
VALUES 
  (1, 1, 0.0),
  (2, 1, 0.0),
  (3, 2, 0.0),
  (1, 3, 100.0),
  (2, 3, 720.0),
  (3, 3, 40.0),
  (3, 4, 0.0),
  (3, 5, 0.0);

-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin
DELETE FROM vehicles;
DELETE FROM trees;
DELETE FROM tree_clusters;
DELETE FROM sensor_data;
DELETE FROM sensors;
DELETE FROM watering_plans;
DELETE FROM vehicle_watering_plans;
DELETE FROM tree_cluster_watering_plans;
-- +goose StatementEnd
