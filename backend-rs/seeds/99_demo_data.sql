INSERT INTO vehicles (id, number_plate, description, water_capacity, type, status, driving_license, model, width, height, length, weight) VALUES
  ('019e3fcb-2a51-7e7e-9392-c4e6dfb61845'::uuid, 'FL-GE-101', 'Kleintransporter für Stadtgebiete', 500.0, 'transporter', 'active', 'B', 'VW Transporter T6.1', 1.90, 1.99, 5.30, 2.0),
  ('019e3fcb-2a51-75fd-818b-913a7cd9cec1'::uuid, 'FL-GE-201', 'Transporter für mittlere Einsätze', 800.0, 'transporter', 'available', 'B', 'Ford Transit Custom', 2.06, 2.52, 5.53, 2.3),
  ('019e3fcb-2a51-73cf-b612-cd0ccdac6f0d'::uuid, 'FL-GE-301', 'Pritschenwagen für Anhängerbetrieb', 300.0, 'transporter', 'active', 'BE', 'MAN TGE 3.180 Pritsche', 2.04, 2.35, 5.99, 3.5),
  ('019e3fcb-2a51-7b0b-9875-8c51aae80c31'::uuid, 'FL-GE-401', 'Mittlerer Tankwagen', 5000.0, 'transporter', 'available', 'C', 'Mercedes Atego 1218', 2.30, 3.20, 7.50, 12.0),
  ('019e3fcb-2a51-728a-8d64-5f8b86cf81ad'::uuid, 'FL-GE-501', 'Großer Tankwagen für Außenbereiche', 10000.0, 'transporter', 'available', 'CE', 'MAN TGS 18.320', 2.55, 3.50, 9.50, 18.0),
  ('019e3fcb-2a52-7c3c-8ef9-faf356e13d59'::uuid, 'FL-GE-A01', 'Kleiner Wassertankanhänger für Stadtgebiete', 1000.0, 'trailer', 'available', 'B', 'Humbaur HA 132513', 1.32, 1.50, 2.50, 0.75),
  ('019e3fcb-2a52-705a-9000-ccd25776cfb9'::uuid, 'FL-GE-A02', 'Mittlerer Wassertankanhänger', 2000.0, 'trailer', 'active', 'BE', 'Unsinn WEB 30', 1.80, 1.80, 4.00, 1.2),
  ('019e3fcb-2a52-773f-9ece-2cc22d45b937'::uuid, 'FL-GE-A03', 'Großer Wassertankanhänger für Außeneinsätze', 4000.0, 'trailer', 'available', 'BE', 'Humbaur HTK 3500', 2.10, 2.00, 5.50, 2.5),
  ('019e3fcb-2a52-78ca-aba1-775ddd5ca945'::uuid, 'FL-TEST-01', '[TEST] Unrealistisch: Viel zu groß und schwer für Routing-Tests', 50000.0, 'transporter', 'available', 'CE', 'Test Mega Truck', 3.50, 5.00, 25.0, 80.0),
  ('019e3fcb-2a52-7572-b3cd-5b59be3d19aa'::uuid, 'FL-TEST-02', '[TEST] Unrealistisch: Viel zu klein für Edge-Case-Tests', 100.0, 'transporter', 'available', 'B', 'Test Mini Van', 1.20, 1.00, 2.0, 0.3),
  ('019e3fcb-2a52-7547-b44d-33e5c74e10cf'::uuid, 'FL-TEST-03', '[TEST] Unrealistisch: Extreme Werte für Grenzwert-Tests', 99999.0, 'transporter', 'not available', 'CE', 'Test Heavy Loader', 4.00, 6.00, 30.0, 150.0),
  ('019e3fcb-2a52-729c-9bb6-10669401291e'::uuid, 'FL-TEST-A01', '[TEST] Unrealistisch: Riesiger Testanhänger', 30000.0, 'trailer', 'available', 'CE', 'Test Giant Trailer', 3.00, 4.00, 15.0, 25.0);


INSERT INTO tree_clusters (id, name, watering_status, moisture_level, region_id, address, description, soil_condition, latitude, longitude, geometry) VALUES
  ('019e3fcb-2a53-7fbf-948d-36836e4f42d1'::uuid, 'Solitüde Strand', 'just watered', 0.85, (SELECT id FROM regions WHERE name = 'Mürwik'), 'Solitüde Strand', 'Alle Bäume am Strand', 'sandig', 54.82128536520703, 9.488152515892045, ST_SetSRID(ST_MakePoint(9.488152515892045, 54.82128536520703), 4326)),
  ('019e3fcb-2a53-7619-aa57-c9b2c9a4aa29'::uuid, 'Sankt-Jürgen-Platz', 'moderate', 0.5, (SELECT id FROM regions WHERE name = 'Mürwik'), 'Ulmenstraße', 'Bäume beim Sankt-Jürgen-Platz', 'schluffig', 54.78805731048199, 9.44400186680097, ST_SetSRID(ST_MakePoint(9.44400186680097, 54.78805731048199), 4326)),
  ('019e3fcb-2a53-7bc9-bc1c-a51e04ef632e'::uuid, 'Flensburger Stadion', 'unknown', 0.7, (SELECT id FROM regions WHERE name = 'Mürwik'), 'Flensburger Stadion', 'Alle Bäume in der Gegend des Stadions in Mürwik', 'schluffig', 54.802163, 9.446398, ST_SetSRID(ST_MakePoint(9.446398, 54.802163), 4326)),
  ('019e3fcb-2a53-7493-a7a1-b194faa409d7'::uuid, 'Campus Hochschule', 'bad', 0.1, (SELECT id FROM regions WHERE name = 'Sandberg'), 'Thomas-Finke Straße', 'Gruppe ist besonders anfällig', 'schluffig', 54.77576059694547, 9.450720736264868, ST_SetSRID(ST_MakePoint(9.450720736264868, 54.77576059694547), 4326)),
  ('019e3fcb-2a53-713d-8cae-4a52d3fd2967'::uuid, 'Mathildenstraße', 'bad', 0.4, (SELECT id FROM regions WHERE name = 'Friesischer Berg'), 'Mathildenstraße', 'Sehr enge Straße und dadurch schlecht zu bewässern.', 'schluffig', 54.78219253876479, 9.423978982828825, ST_SetSRID(ST_MakePoint(9.423978982828825, 54.78219253876479), 4326)),
  ('019e3fcb-2a53-7706-a04c-50d6c926c7f8'::uuid, 'Nordstadt', 'good', 0.8, (SELECT id FROM regions WHERE name = 'Nordstadt'), 'Apenrader Straße', 'Guter Baumbestand mit großen Kronen.', 'sandig', 54.807162, 9.423138, ST_SetSRID(ST_MakePoint(9.423138, 54.807162), 4326)),
  ('019e3fcb-2a53-7963-8228-705689f884b9'::uuid, 'TSB Neustadt', 'just watered', 0.9, (SELECT id FROM regions WHERE name = 'Nordstadt'), 'Ecknerstraße', 'Kleiner Baumbestand.', 'sandig', 54.797162, 9.419620, ST_SetSRID(ST_MakePoint(9.419620, 54.797162), 4326)),
  ('019e3fcb-2a53-7be5-9a56-e2260bb1c9fb'::uuid, 'Seniorenanlage Valentinerhof', 'bad', 0.15, (SELECT id FROM regions WHERE name = 'Nordstadt'), 'Auf dem Geländer der Seniorenanlage', 'Sehr viel versiegelter Boden.', 'sandig', 54.76994251235151, 9.441111747447234, ST_SetSRID(ST_MakePoint(9.441111747447234, 54.76994251235151), 4326)),
  ('019e3fcb-2a53-7433-8d57-6258f9688c82'::uuid, 'Peelwatt', 'unknown', 0.3, (SELECT id FROM regions WHERE name = 'Nordstadt'), 'Peelwatt halt', 'Sehr viel versiegelter Boden.', 'sandig', 54.76671656688957, 9.456136954289867, ST_SetSRID(ST_MakePoint(9.456136954289867, 54.76671656688957), 4326)),
  ('019e3fcb-2a53-7fc1-bc32-9c8885df5027'::uuid, 'Lautrupsbach', 'moderate', 0.45, (SELECT id FROM regions WHERE name = 'Nordstadt'), 'An der Nordstraße', 'Sehr viel versiegelter Boden.', 'sandig', 54.79265065021804, 9.454269041383837, ST_SetSRID(ST_MakePoint(9.454269041383837, 54.76671656688957), 4326));


INSERT INTO sensors (id, status, type, model_id)
SELECT v.id, v.status::sensor_status, v.type::sensor_type, (SELECT id FROM sensor_models WHERE name = 'EcoDrizzler')
FROM (VALUES
    ('sensor-1', 'offline', 'lorawan'),
    ('sensor-2', 'offline', 'lorawan'),
    ('sensor-3', 'offline', 'lorawan'),
    ('sensor-4', 'offline', 'lorawan'),
    ('sensor-5', 'offline', 'lorawan'),
    ('sensor-6', 'offline', 'lorawan'),
    ('sensor-7', 'offline', 'lorawan'),
    ('sensor-8', 'offline', 'lorawan')
) AS v(id, status, type);

INSERT INTO sensor_lorawan (id, serial_number, dev_eui, app_eui, app_key)
VALUES
    ('sensor-1', '', '', '', ''),
    ('sensor-2', '', '', '', ''),
    ('sensor-3', '', '', '', ''),
    ('sensor-4', '', '', '', ''),
    ('sensor-5', '', '', '', ''),
    ('sensor-6', '', '', '', ''),
    ('sensor-7', '', '', '', ''),
    ('sensor-8', '', '', '', '');

INSERT INTO trees (id, tree_cluster_id, sensor_id, planting_year, species, number, latitude, longitude, geometry, watering_status, description) VALUES
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7fbf-948d-36836e4f42d1'::uuid, 'sensor-1', 2023, 'Quercus robur', 1005, 54.82124518093376, 9.485702120628517, ST_SetSRID(ST_MakePoint(9.485702120628517, 54.82124518093376), 4326), 'good', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7fbf-948d-36836e4f42d1'::uuid, 'sensor-2', 2023, 'Quercus robur', 1006, 54.8215076622281, 9.487153277881877, ST_SetSRID(ST_MakePoint(9.487153277881877, 54.8215076622281), 4326), 'good', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7fbf-948d-36836e4f42d1'::uuid, NULL, 2023, 'Quercus robur', 1007, 54.82078826498143, 9.489684366114483, ST_SetSRID(ST_MakePoint(9.489684366114483, 54.82078826498143), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7fbf-948d-36836e4f42d1'::uuid, NULL, 2023, 'Quercus robur', 1001, 54.820834078576304, 9.486398528109389, ST_SetSRID(ST_MakePoint(9.486398528109389, 54.820834078576304), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7fbf-948d-36836e4f42d1'::uuid, NULL, 2023, 'Quercus robur', 1002, 54.82008971976509, 9.488979617332221, ST_SetSRID(ST_MakePoint(9.488979617332221, 54.82008971976509), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7fbf-948d-36836e4f42d1'::uuid, NULL, 2023, 'Quercus robur', 1003, 54.82061210171266, 9.486168703385617, ST_SetSRID(ST_MakePoint(9.486168703385617, 54.82061210171266), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7fbf-948d-36836e4f42d1'::uuid, NULL, 2023, 'Quercus robur', 1004, 54.8199067243877, 9.487106513347264, ST_SetSRID(ST_MakePoint(9.487106513347264, 54.8199067243877), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7fbf-948d-36836e4f42d1'::uuid, NULL, 2023, 'Quercus robur', 2001, 54.821248829192285, 9.48996664076417, ST_SetSRID(ST_MakePoint(9.48996664076417, 54.821248829192285), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7619-aa57-c9b2c9a4aa29'::uuid, 'sensor-3', 2022, 'Quercus robur', 1008, 54.78780993841013, 9.444052105200551, ST_SetSRID(ST_MakePoint(9.444052105200551, 54.78780993841013), 4326), 'moderate', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7619-aa57-c9b2c9a4aa29'::uuid, NULL, 2022, 'Quercus robur', 1009, 54.78836553796373, 9.444075995492044, ST_SetSRID(ST_MakePoint(9.444075995492044, 54.78836553796373), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7619-aa57-c9b2c9a4aa29'::uuid, NULL, 2022, 'Quercus robur', 1010, 54.787768612518455, 9.443996361187065, ST_SetSRID(ST_MakePoint(9.443996361187065, 54.787768612518455), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7619-aa57-c9b2c9a4aa29'::uuid, NULL, 2022, 'Quercus robur', 1010, 54.78826721846835, 9.443595915277797, ST_SetSRID(ST_MakePoint(9.443595915277797, 54.78826721846835), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7619-aa57-c9b2c9a4aa29'::uuid, NULL, 2022, 'Quercus robur', 1010, 54.78810634901004, 9.44443262510434, ST_SetSRID(ST_MakePoint(9.44443262510434, 54.78810634901004), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7619-aa57-c9b2c9a4aa29'::uuid, NULL, 2022, 'Quercus robur', 1010, 54.78815894101875, 9.443955271421238, ST_SetSRID(ST_MakePoint(9.443955271421238, 54.78815894101875), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7bc9-bc1c-a51e04ef632e'::uuid, NULL, 2023, 'Betula pendula', 1034, 54.801718, 9.444797, ST_SetSRID(ST_MakePoint(9.444797, 54.801718), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7bc9-bc1c-a51e04ef632e'::uuid, NULL, 2023, 'Betula pendula', 1035, 54.800797, 9.444271, ST_SetSRID(ST_MakePoint(9.444271, 54.800797), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7bc9-bc1c-a51e04ef632e'::uuid, NULL, 2023, 'Betula pendula', 1036, 54.801539, 9.446741, ST_SetSRID(ST_MakePoint(9.446741, 54.801539), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7bc9-bc1c-a51e04ef632e'::uuid, NULL, 2023, 'Betula pendula', 1037, 54.799796, 9.443927, ST_SetSRID(ST_MakePoint(9.443927, 54.799796), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7bc9-bc1c-a51e04ef632e'::uuid, NULL, 2023, 'Betula pendula', 1038, 54.804052, 9.447900, ST_SetSRID(ST_MakePoint(9.447900, 54.804052), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7493-a7a1-b194faa409d7'::uuid, 'sensor-4', 2022, 'Tilia intermedia', 1029, 54.775679885633636, 9.451171073968197, ST_SetSRID(ST_MakePoint(9.451171073968197, 54.775679885633636), 4326), 'bad', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7493-a7a1-b194faa409d7'::uuid, NULL, 2022, 'Tilia intermedia', 1027, 54.776120, 9.450891, ST_SetSRID(ST_MakePoint(9.450891, 54.776120), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7493-a7a1-b194faa409d7'::uuid, NULL, 2022, 'Tilia intermedia', 1028, 54.776058, 9.450311, ST_SetSRID(ST_MakePoint(9.450311, 54.776058), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7493-a7a1-b194faa409d7'::uuid, NULL, 2022, 'Tilia intermedia', 1029, 54.775709, 9.447762, ST_SetSRID(ST_MakePoint(9.447762, 54.775709), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7493-a7a1-b194faa409d7'::uuid, NULL, 2022, 'Tilia intermedia', 1026, 54.776145, 9.449785, ST_SetSRID(ST_MakePoint(9.449785, 54.776145), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7493-a7a1-b194faa409d7'::uuid, NULL, 2022, 'Tilia intermedia', 1026, 54.774986825456224, 9.451846963834953, ST_SetSRID(ST_MakePoint(9.451846963834953, 54.774986825456224), 4326), 'unknown', 'UNSER TEST BAUM'),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-713d-8cae-4a52d3fd2967'::uuid, 'sensor-5', 2021, 'Fraxinus ornus Obelisk', 1021, 54.782630, 9.423792, ST_SetSRID(ST_MakePoint(9.423792, 54.782630), 4326), 'bad', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-713d-8cae-4a52d3fd2967'::uuid, NULL, 2021, 'Fraxinus ornus Obelisk', 1022, 54.782463, 9.423727, ST_SetSRID(ST_MakePoint(9.423727, 54.782463), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-713d-8cae-4a52d3fd2967'::uuid, NULL, 2021, 'Fraxinus ornus Obelisk', 1023, 54.782296, 9.424178, ST_SetSRID(ST_MakePoint(9.424178, 54.782296), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-713d-8cae-4a52d3fd2967'::uuid, NULL, 2022, 'Fraxinus ornus Obelisk', 1024, 54.782043, 9.424188, ST_SetSRID(ST_MakePoint(9.424188, 54.782043), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-713d-8cae-4a52d3fd2967'::uuid, NULL, 2022, 'Fraxinus ornus Obelisk', 1025, 54.781753, 9.424936, ST_SetSRID(ST_MakePoint(9.424936, 54.781753), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7706-a04c-50d6c926c7f8'::uuid, NULL, 2023, 'Acer platanoides Schwedleri', 1039, 54.806287, 9.423469, ST_SetSRID(ST_MakePoint(9.423469, 54.806287), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7706-a04c-50d6c926c7f8'::uuid, NULL, 2023, 'Acer platanoides Schwedleri', 1040, 54.807212, 9.422752, ST_SetSRID(ST_MakePoint(9.422752, 54.807212), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7706-a04c-50d6c926c7f8'::uuid, NULL, 2023, 'Acer platanoides Schwedleri', 1041, 54.806606, 9.422773, ST_SetSRID(ST_MakePoint(9.422773, 54.806606), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7706-a04c-50d6c926c7f8'::uuid, NULL, 2023, 'Acer platanoides Schwedleri', 1042, 54.807787, 9.422354, ST_SetSRID(ST_MakePoint(9.422354, 54.807787), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7963-8228-705689f884b9'::uuid, 'sensor-6', 2022, 'Acer platanoides Schwedleri', 1043, 54.796916, 9.421332, ST_SetSRID(ST_MakePoint(9.421332, 54.796916), 4326), 'good', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7963-8228-705689f884b9'::uuid, NULL, 2022, 'Acer platanoides Schwedleri', 1044, 54.797330, 9.419002, ST_SetSRID(ST_MakePoint(9.419002, 54.797330), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7963-8228-705689f884b9'::uuid, NULL, 2022, 'Acer platanoides Schwedleri', 1045, 54.797114, 9.417843, ST_SetSRID(ST_MakePoint(9.417843, 54.797114), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7be5-9a56-e2260bb1c9fb'::uuid, 'sensor-7', 2023, 'Populus cf. suaveolens', 1052, 54.7697451282801, 9.439562555553788, ST_SetSRID(ST_MakePoint(9.439562555553788, 54.7697451282801), 4326), 'bad', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7be5-9a56-e2260bb1c9fb'::uuid, NULL, 2023, 'Populus cf. suaveolens', 5555, 54.76932352301634, 9.441299419876234, ST_SetSRID(ST_MakePoint(9.441299419876234, 54.76932352301634), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7be5-9a56-e2260bb1c9fb'::uuid, NULL, 2023, 'Populus cf. suaveolens', 4444, 54.76915329290317, 9.441851862902759, ST_SetSRID(ST_MakePoint(9.441851862902759, 54.76915329290317), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7be5-9a56-e2260bb1c9fb'::uuid, NULL, 2023, 'Populus cf. suaveolens', 3333, 54.770304653528044, 9.44233994363491, ST_SetSRID(ST_MakePoint(9.44233994363491, 54.770304653528044), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7be5-9a56-e2260bb1c9fb'::uuid, NULL, 2023, 'Populus cf. suaveolens', 2222, 54.771043653535294, 9.440740347234932, ST_SetSRID(ST_MakePoint(9.440740347234932, 54.771043653535294), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7433-8d57-6258f9688c82'::uuid, NULL, 2024, 'Tilia x vulgaris', 1015, 54.76752937879732, 9.457372632491829, ST_SetSRID(ST_MakePoint(9.457372632491829, 54.76752937879732), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7433-8d57-6258f9688c82'::uuid, NULL, 2024, 'Tilia x vulgaris', 1015, 54.767564688002714, 9.453443844886783, ST_SetSRID(ST_MakePoint(9.453443844886783, 54.767564688002714), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7433-8d57-6258f9688c82'::uuid, NULL, 2024, 'Tilia x vulgaris', 1015, 54.765620842535895, 9.4575523046762, ST_SetSRID(ST_MakePoint(9.4575523046762, 54.765620842535895), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7433-8d57-6258f9688c82'::uuid, NULL, 2024, 'Tilia x vulgaris', 1015, 54.76725516472003, 9.456833592389275, ST_SetSRID(ST_MakePoint(9.456833592389275, 54.76725516472003), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7fc1-bc32-9c8885df5027'::uuid, 'sensor-8', 2023, 'Alnus glutinosa', 1030, 54.792472, 9.452773, ST_SetSRID(ST_MakePoint(9.452773, 54.792472), 4326), 'moderate', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7fc1-bc32-9c8885df5027'::uuid, NULL, 2023, 'Alnus glutinosa', 1031, 54.792782, 9.453795, ST_SetSRID(ST_MakePoint(9.453795, 54.792782), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7fc1-bc32-9c8885df5027'::uuid, NULL, 2023, 'Alnus glutinosa', 1032, 54.792837, 9.454880, ST_SetSRID(ST_MakePoint(9.454880, 54.792837), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), '019e3fcb-2a53-7fc1-bc32-9c8885df5027'::uuid, NULL, 2023, 'Alnus glutinosa', 1033, 54.792435, 9.455545, ST_SetSRID(ST_MakePoint(9.455545, 54.792435), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), NULL, NULL, 2024, 'Carpinus betulus', 1015, 54.783739, 9.426823, ST_SetSRID(ST_MakePoint(9.426823, 54.783739), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), NULL, NULL, 2022, 'Carpinus betulus', 1017, 54.785981, 9.430668, ST_SetSRID(ST_MakePoint(9.430668, 54.785981), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), NULL, NULL, 2022, 'Carpinus betulus', 1018, 54.786269, 9.431758, ST_SetSRID(ST_MakePoint(9.431758, 54.786269), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), NULL, NULL, 2022, 'Carpinus betulus', 1019, 54.787339, 9.431701, ST_SetSRID(ST_MakePoint(9.431701, 54.787339), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), NULL, NULL, 2021, 'Carpinus betulus', 1020, 54.786656, 9.432243, ST_SetSRID(ST_MakePoint(9.432243, 54.786656), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (uuidv7_from_timestamp(now()::timestamp), NULL, NULL, 2022, 'Populus cf. suaveolens', 1051, 54.769030, 9.429936, ST_SetSRID(ST_MakePoint(9.429936, 54.769030), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), NULL, NULL, 2021, 'Populus cf. suaveolens', 1053, 54.775237, 9.441981, ST_SetSRID(ST_MakePoint(9.441981, 54.775237), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), NULL, NULL, 2022, 'Populus cf. suaveolens', 1054, 54.780192, 9.459607, ST_SetSRID(ST_MakePoint(9.459607, 54.780192), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (uuidv7_from_timestamp(now()::timestamp), NULL, NULL, 2021, 'Populus cf. suaveolens', 1055, 54.785043, 9.418210, ST_SetSRID(ST_MakePoint(9.418210, 54.785043), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), NULL, NULL, 2022, 'Fraxinus excelsior', 1056, 54.779697, 9.440026, ST_SetSRID(ST_MakePoint(9.440026, 54.779697), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), NULL, NULL, 2020, 'Fraxinus excelsior', 1057, 54.785147, 9.438903, ST_SetSRID(ST_MakePoint(9.438903, 54.785147), 4326), 'unknown', ''),
  (uuidv7_from_timestamp(now()::timestamp), NULL, NULL, 2020, 'Fraxinus excelsior', 1058, 54.788205, 9.454699, ST_SetSRID(ST_MakePoint(9.454699, 54.788205), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (uuidv7_from_timestamp(now()::timestamp), NULL, NULL, 2020, 'Fraxinus excelsior', 1059, 54.804054, 9.469544, ST_SetSRID(ST_MakePoint(9.469544, 54.804054), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (uuidv7_from_timestamp(now()::timestamp), NULL, NULL, 2024, 'Acer pseudoplatanus', 1060, 54.813655, 9.477633, ST_SetSRID(ST_MakePoint(9.477633, 54.813655), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (uuidv7_from_timestamp(now()::timestamp), NULL, NULL, 2024, 'Acer pseudoplatanus', 1061, 54.811001, 9.484132, ST_SetSRID(ST_MakePoint(9.484132, 54.811001), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt'),
  (uuidv7_from_timestamp(now()::timestamp), NULL, NULL, 2024, 'Acer pseudoplatanus', 1062, 54.790366, 9.472744, ST_SetSRID(ST_MakePoint(9.472744, 54.790366), 4326), 'unknown', '');


INSERT INTO trees (id, tree_cluster_id, sensor_id, planting_year, species, number, latitude, longitude, geometry, watering_status, description, last_watered) VALUES
  (uuidv7_from_timestamp(now()::timestamp), NULL, NULL, 2023, 'Quercus robur', 1065, 54.780394213230196, 9.417514801025392, ST_SetSRID(ST_MakePoint(9.417514801025392, 54.780394213230196), 4326), 'unknown', 'Dieser Baum wurde im August das letzte mal gestuzt', '2025-02-14 12:34:56');


INSERT INTO sensor_data (id, sensor_id, data) VALUES
  (uuidv7_from_timestamp(now()::timestamp), 'sensor-1', '{
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
  (uuidv7_from_timestamp(now()::timestamp), 'sensor-1', '{
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
  (uuidv7_from_timestamp(now()::timestamp), 'sensor-2', '{
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
  (uuidv7_from_timestamp(now()::timestamp), 'sensor-2', '{
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
  (uuidv7_from_timestamp(now()::timestamp), 'sensor-2', '{
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
  (uuidv7_from_timestamp(now()::timestamp), 'sensor-2', '{
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
  (uuidv7_from_timestamp(now()::timestamp), 'sensor-3', '{
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
  (uuidv7_from_timestamp(now()::timestamp), 'sensor-3', '{
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
  (uuidv7_from_timestamp(now()::timestamp), 'sensor-4', '{
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
  (uuidv7_from_timestamp(now()::timestamp), 'sensor-4', '{
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
  (uuidv7_from_timestamp(now()::timestamp), 'sensor-4', '{
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
  (uuidv7_from_timestamp(now()::timestamp), 'sensor-5', '{
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
  (uuidv7_from_timestamp(now()::timestamp), 'sensor-5', '{
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
  (uuidv7_from_timestamp(now()::timestamp), 'sensor-6', '{
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
  (uuidv7_from_timestamp(now()::timestamp), 'sensor-7', '{
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
  (uuidv7_from_timestamp(now()::timestamp), 'sensor-7', '{
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
  (uuidv7_from_timestamp(now()::timestamp), 'sensor-8', '{
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
  (uuidv7_from_timestamp(now()::timestamp), 'sensor-8', '{
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


INSERT INTO watering_plans (id, date, description, status, distance, total_water_required, cancellation_note) VALUES
  ('019e3fcb-2a5c-70b8-b5c4-fb729bd9c877'::uuid, '2025-09-22', 'New watering plan for the west side of the city', 'planned', 63.0, 720.0, ''),
  ('019e3fcb-2a5d-7e82-afe5-626abd96274d'::uuid, '2025-08-03', 'New watering plan for the east side of the city', 'active', 63.0, 600.0, ''),
  ('019e3fcb-2a5d-7c0d-bbd9-7b3d84f5c5f3'::uuid, '2025-06-12', 'Very important watering plan due to no rainfall', 'finished', 63.0, 1320.0, ''),
  ('019e3fcb-2a5d-7e3b-a202-44acf2ffcb71'::uuid, '2025-06-10', 'New watering plan for the south side of the city', 'not competed', 63.0, 600.0, ''),
  ('019e3fcb-2a5d-7e0d-840e-fe831360cfdb'::uuid, '2025-06-04', 'Canceled due to flood', 'canceled', 63.0, 600.0, 'The watering plan was cancelled due to various reasons.');


INSERT INTO vehicle_watering_plans (vehicle_id, watering_plan_id) VALUES
  ('019e3fcb-2a51-7e7e-9392-c4e6dfb61845'::uuid, '019e3fcb-2a5c-70b8-b5c4-fb729bd9c877'::uuid),
  ('019e3fcb-2a51-75fd-818b-913a7cd9cec1'::uuid, '019e3fcb-2a5c-70b8-b5c4-fb729bd9c877'::uuid),
  ('019e3fcb-2a51-75fd-818b-913a7cd9cec1'::uuid, '019e3fcb-2a5d-7e82-afe5-626abd96274d'::uuid),
  ('019e3fcb-2a51-75fd-818b-913a7cd9cec1'::uuid, '019e3fcb-2a5d-7c0d-bbd9-7b3d84f5c5f3'::uuid),
  ('019e3fcb-2a51-75fd-818b-913a7cd9cec1'::uuid, '019e3fcb-2a5d-7e3b-a202-44acf2ffcb71'::uuid),
  ('019e3fcb-2a51-75fd-818b-913a7cd9cec1'::uuid, '019e3fcb-2a5d-7e0d-840e-fe831360cfdb'::uuid);


INSERT INTO user_watering_plans (user_id, watering_plan_id) VALUES
  ('6a1078e8-80fd-458f-b74e-e388fe2dd6ab', '019e3fcb-2a5c-70b8-b5c4-fb729bd9c877'::uuid),
  ('6a1078e8-80fd-458f-b74e-e388fe2dd6ab', '019e3fcb-2a5d-7e82-afe5-626abd96274d'::uuid),
  ('6a1078e8-80fd-458f-b74e-e388fe2dd6ab', '019e3fcb-2a5d-7c0d-bbd9-7b3d84f5c5f3'::uuid),
  ('6a1078e8-80fd-458f-b74e-e388fe2dd6ab', '019e3fcb-2a5d-7e3b-a202-44acf2ffcb71'::uuid),
  ('6a1078e8-80fd-458f-b74e-e388fe2dd6ab', '019e3fcb-2a5d-7e0d-840e-fe831360cfdb'::uuid);


INSERT INTO tree_cluster_watering_plans (tree_cluster_id, watering_plan_id, consumed_water) VALUES
  ('019e3fcb-2a53-7fbf-948d-36836e4f42d1'::uuid, '019e3fcb-2a5c-70b8-b5c4-fb729bd9c877'::uuid, 0.0),
  ('019e3fcb-2a53-7619-aa57-c9b2c9a4aa29'::uuid, '019e3fcb-2a5c-70b8-b5c4-fb729bd9c877'::uuid, 0.0),
  ('019e3fcb-2a53-7bc9-bc1c-a51e04ef632e'::uuid, '019e3fcb-2a5d-7e82-afe5-626abd96274d'::uuid, 0.0),
  ('019e3fcb-2a53-7fbf-948d-36836e4f42d1'::uuid, '019e3fcb-2a5d-7c0d-bbd9-7b3d84f5c5f3'::uuid, 100.0),
  ('019e3fcb-2a53-7619-aa57-c9b2c9a4aa29'::uuid, '019e3fcb-2a5d-7c0d-bbd9-7b3d84f5c5f3'::uuid, 720.0),
  ('019e3fcb-2a53-7bc9-bc1c-a51e04ef632e'::uuid, '019e3fcb-2a5d-7c0d-bbd9-7b3d84f5c5f3'::uuid, 40.0),
  ('019e3fcb-2a53-7bc9-bc1c-a51e04ef632e'::uuid, '019e3fcb-2a5d-7e3b-a202-44acf2ffcb71'::uuid, 0.0),
  ('019e3fcb-2a53-7bc9-bc1c-a51e04ef632e'::uuid, '019e3fcb-2a5d-7e0d-840e-fe831360cfdb'::uuid, 0.0);


-- Safety net: rebuild geometry from lat/lng columns in case a future row is added
-- with the wrong ST_MakePoint argument order (PostGIS expects lng, lat).
UPDATE tree_clusters
   SET geometry = ST_SetSRID(ST_MakePoint(longitude, latitude), 4326)
 WHERE longitude IS NOT NULL AND latitude IS NOT NULL;

UPDATE trees
   SET geometry = ST_SetSRID(ST_MakePoint(longitude, latitude), 4326)
 WHERE longitude IS NOT NULL AND latitude IS NOT NULL;


-- Prepared sensors (GES-1000) to exercise the activate flow during demos.

INSERT INTO sensors (id, status, type, model_id)
SELECT v.id, v.status::sensor_status, v.type::sensor_type, (SELECT id FROM sensor_models WHERE name = 'GES-1000')
FROM (VALUES
    ('sensor-prepared-1', 'prepared', 'lorawan'),
    ('sensor-prepared-2', 'prepared', 'lorawan')
) AS v(id, status, type);

INSERT INTO sensor_lorawan (id, serial_number, dev_eui, app_eui, app_key)
VALUES
    ('sensor-prepared-1', '', '', '', ''),
    ('sensor-prepared-2', '', '', '', '');