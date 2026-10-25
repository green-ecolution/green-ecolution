INSERT INTO organization_settings (organization_id, water_demand_liters, just_watered_ttl_secs)
VALUES ('01980000-0000-7000-8000-000000000001', 55, 7200);

INSERT INTO sensors (id, model_id, type, organization_id, activated_at)
VALUES (
    'eui-migration-1',
    (SELECT id FROM sensor_models WHERE name = 'EcoDrizzler'),
    'lorawan',
    '01980000-0000-7000-8000-000000000001',
    now()
);

INSERT INTO tree_clusters (id, name, address, description, moisture_level, organization_id)
VALUES (
    '01980000-0000-7000-8000-000000000c01',
    'Migrationscluster',
    'Teststrasse 1',
    'seeded before stage 3',
    0.4,
    '01980000-0000-7000-8000-000000000001'
);

INSERT INTO trees (id, tree_cluster_id, sensor_id, number, species, planting_year, latitude, longitude, geometry, organization_id)
VALUES (
    '01980000-0000-7000-8000-000000000e01',
    '01980000-0000-7000-8000-000000000c01',
    'eui-migration-1',
    'MIG-001',
    'Quercus robur',
    2020,
    54.79,
    9.44,
    ST_SetSRID(ST_MakePoint(9.44, 54.79), 4326),
    '01980000-0000-7000-8000-000000000001'
);

INSERT INTO vehicles (id, number_plate, model, description, type, water_capacity, organization_id)
VALUES (
    '01980000-0000-7000-8000-000000000a01',
    'FL-MIG 1',
    'Transporter',
    'seeded before stage 3',
    'transporter',
    3000,
    '01980000-0000-7000-8000-000000000001'
);

INSERT INTO depots (id, name, watering_point, latitude, longitude, geometry, organization_id)
VALUES (
    '01980000-0000-7000-8000-000000000d01',
    'Migrationsdepot',
    true,
    54.78,
    9.43,
    ST_SetSRID(ST_MakePoint(9.43, 54.78), 4326),
    '01980000-0000-7000-8000-000000000001'
);

INSERT INTO watering_plans (id, date, status, description, organization_id)
VALUES
    ('01980000-0000-7000-8000-000000000f01', '2026-10-01', 'finished', 'finished plan', '01980000-0000-7000-8000-000000000001'),
    ('01980000-0000-7000-8000-000000000f02', '2026-10-02', 'planned', 'planned plan', '01980000-0000-7000-8000-000000000001');

INSERT INTO tree_cluster_watering_plans (tree_cluster_id, watering_plan_id, consumed_water)
VALUES
    ('01980000-0000-7000-8000-000000000c01', '01980000-0000-7000-8000-000000000f01', 140),
    ('01980000-0000-7000-8000-000000000c01', '01980000-0000-7000-8000-000000000f02', 0);

INSERT INTO vehicle_watering_plans (vehicle_id, watering_plan_id, role)
VALUES
    ('01980000-0000-7000-8000-000000000a01', '01980000-0000-7000-8000-000000000f01', 'transporter'),
    ('01980000-0000-7000-8000-000000000a01', '01980000-0000-7000-8000-000000000f02', 'transporter');

INSERT INTO comments (id, subject_type, subject_id, author_id, body)
VALUES
    ('01980000-0000-7000-8000-000000000b01', 'tree_cluster', '01980000-0000-7000-8000-000000000c01', '01980000-0000-7000-8000-0000000000aa', 'cluster comment'),
    ('01980000-0000-7000-8000-000000000b02', 'watering_plan', '01980000-0000-7000-8000-000000000f01', '01980000-0000-7000-8000-0000000000aa', 'plan comment');
