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
