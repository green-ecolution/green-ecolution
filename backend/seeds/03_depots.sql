INSERT INTO depots (id, name, latitude, longitude, geometry, replenishes, is_default, organization_id) VALUES
  (gen_random_uuid(), 'Betriebshof Schleswiger Straße', 54.76879146396569, 9.434803531218018, ST_SetSRID(ST_MakePoint(9.434803531218018, 54.76879146396569), 4326), '{water}', TRUE, '01980000-0000-7000-8000-000000000002'),
  (gen_random_uuid(), 'Klärwerk Kielseng', 54.80518123149477, 9.447145106541388, ST_SetSRID(ST_MakePoint(9.447145106541388, 54.80518123149477), 4326), '{water,cargo_volume}', FALSE, '01980000-0000-7000-8000-000000000002');
