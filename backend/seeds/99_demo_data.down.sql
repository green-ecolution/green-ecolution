DELETE FROM role_assignments WHERE user_id IN (
    '8e1a16e9-19b6-4bcc-a5a3-3e6fa7518865',
    '6a1078e8-80fd-458f-b74e-e388fe2dd6ab',
    'b5afc591-ee33-4df8-af75-265265f05882'
);
DELETE FROM user_profiles WHERE id IN (
    '8e1a16e9-19b6-4bcc-a5a3-3e6fa7518865',
    '6a1078e8-80fd-458f-b74e-e388fe2dd6ab',
    'b5afc591-ee33-4df8-af75-265265f05882'
);
DELETE FROM vehicles;
DELETE FROM trees;
DELETE FROM tree_clusters;
DELETE FROM sensor_data;
DELETE FROM sensor_lorawan WHERE id IN (
    'sensor-1', 'sensor-2', 'sensor-3', 'sensor-4',
    'sensor-5', 'sensor-6', 'sensor-7', 'sensor-8',
    'sensor-prepared-1', 'sensor-prepared-2'
);
DELETE FROM sensors WHERE id IN (
    'sensor-1', 'sensor-2', 'sensor-3', 'sensor-4',
    'sensor-5', 'sensor-6', 'sensor-7', 'sensor-8',
    'sensor-prepared-1', 'sensor-prepared-2'
);
DELETE FROM watering_plans;
DELETE FROM vehicle_watering_plans;
DELETE FROM tree_cluster_watering_plans;
