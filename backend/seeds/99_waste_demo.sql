-- Demo data for the waste module: bins around Flensburg in every fill state,
-- simulated distance sensors with a short history, a collection vehicle and
-- one collection tour per board column. Runs after 99_demo_data.sql because
-- the tours reference its users.

-- The module ships switched off; TBZ switches it on for itself and its
-- sub-organizations (set_by = TBZ, so TBZ may reset it).
INSERT INTO organization_modules (organization_id, module_key, enabled, set_by) VALUES
  ('01980000-0000-7000-8000-000000000002', 'waste', TRUE, '01980000-0000-7000-8000-000000000002')
ON CONFLICT (organization_id, module_key) DO NOTHING;

INSERT INTO vehicles (id, number_plate, description, type, availability, driving_license, model, width, height, length, weight, organization_id) VALUES
  ('019e3fcb-2a51-7601-8000-000000000601'::uuid, 'FL-GE-601', 'Abfallsammelfahrzeug für Innenstadt und Stadtteile', 'transporter', 'available', 'C', 'Mercedes Econic 2630', 2.55, 3.40, 9.30, 26.0, '01980000-0000-7000-8000-000000000002');

INSERT INTO vehicle_capacities (vehicle_id, kind, liters) VALUES
  ('019e3fcb-2a51-7601-8000-000000000601'::uuid, 'cargo_volume', 16000.0),
  -- the flatbed from 99_demo_data.sql can take a few bins as well
  ('019e3fcb-2a51-73cf-b612-cd0ccdac6f0d'::uuid, 'cargo_volume', 3000.0);

-- depth_cm is the distance from the sensor to the bottom of an empty bin.
INSERT INTO waste_bins (id, number, latitude, longitude, geometry, volume_liters, depth_cm, organization_id, additional_informations) VALUES
  ('019e3fcb-2a5f-7001-8000-000000000001'::uuid, 'B-001', 54.79330, 9.43325, ST_SetSRID(ST_MakePoint(9.43325, 54.79330), 4326), 1100.0, 120, '01980000-0000-7000-8000-000000000002', '{"location": "Schiffbrücke, Museumshafen"}'),
  ('019e3fcb-2a5f-7001-8000-000000000002'::uuid, 'B-002', 54.78189, 9.43374, ST_SetSRID(ST_MakePoint(9.43374, 54.78189), 4326),  240.0, 100, '01980000-0000-7000-8000-000000000002', '{"location": "Südermarkt"}'),
  ('019e3fcb-2a5f-7001-8000-000000000003'::uuid, 'B-003', 54.79395, 9.43151, ST_SetSRID(ST_MakePoint(9.43151, 54.79395), 4326),  240.0, 100, '01980000-0000-7000-8000-000000000002', '{"location": "Nordertor"}'),
  ('019e3fcb-2a5f-7001-8000-000000000004'::uuid, 'B-004', 54.78512, 9.43810, ST_SetSRID(ST_MakePoint(9.43810, 54.78512), 4326), 1100.0, 120, '01980000-0000-7000-8000-000000000002', '{"location": "ZOB, Bussteig A"}'),
  ('019e3fcb-2a5f-7001-8000-000000000005'::uuid, 'B-005', 54.77280, 9.42930, ST_SetSRID(ST_MakePoint(9.42930, 54.77280), 4326),  240.0, 100, '01980000-0000-7000-8000-000000000002', '{"location": "Volkspark, Haupteingang"}'),
  ('019e3fcb-2a5f-7001-8000-000000000006'::uuid, 'B-006', 54.78329, 9.42911, ST_SetSRID(ST_MakePoint(9.42911, 54.78329), 4326), 1100.0, 120, '01980000-0000-7000-8000-000000000002', '{"location": "Museumsberg"}'),
  ('019e3fcb-2a5f-7001-8000-000000000007'::uuid, 'B-007', 54.81752, 9.48300, ST_SetSRID(ST_MakePoint(9.48300, 54.81752), 4326),  240.0, 100, '01980000-0000-7000-8000-000000000002', '{"location": "Strandbad Solitüde"}'),
  ('019e3fcb-2a5f-7001-8000-000000000008'::uuid, 'B-008', 54.77568, 9.45120, ST_SetSRID(ST_MakePoint(9.45120, 54.77568), 4326),  240.0, 100, '01980000-0000-7000-8000-000000000003', '{"location": "Campus Hochschule, Mensa"}');

-- B-005 stays without a sensor, sensor-bin-6 is attached but has not sent
-- anything yet (B-007), so both show up as "Unbekannt" for different reasons.
INSERT INTO sensors (id, activated_at, type, model_id, organization_id)
SELECT v.id, CURRENT_TIMESTAMP - interval '30 days', 'lorawan'::sensor_type, (SELECT id FROM sensor_models WHERE name = 'GE Distance (simuliert)'), v.org_id::uuid
FROM (VALUES
    ('sensor-bin-1', '01980000-0000-7000-8000-000000000002'),
    ('sensor-bin-2', '01980000-0000-7000-8000-000000000002'),
    ('sensor-bin-3', '01980000-0000-7000-8000-000000000002'),
    ('sensor-bin-4', '01980000-0000-7000-8000-000000000002'),
    ('sensor-bin-5', '01980000-0000-7000-8000-000000000002'),
    ('sensor-bin-6', '01980000-0000-7000-8000-000000000002'),
    ('sensor-bin-7', '01980000-0000-7000-8000-000000000003')
) AS v(id, org_id);

INSERT INTO sensor_lorawan (id, serial_number, dev_eui, app_eui, app_key) VALUES
    ('sensor-bin-1', 'GED-2026-0001', '0004A30B00AC0001', '0004A30B00AC0000', '3C1F6A9E2B7D4E8F90A1B2C3D4E5F601'),
    ('sensor-bin-2', 'GED-2026-0002', '0004A30B00AC0002', '0004A30B00AC0000', '3C1F6A9E2B7D4E8F90A1B2C3D4E5F602'),
    ('sensor-bin-3', 'GED-2026-0003', '0004A30B00AC0003', '0004A30B00AC0000', '3C1F6A9E2B7D4E8F90A1B2C3D4E5F603'),
    ('sensor-bin-4', 'GED-2026-0004', '0004A30B00AC0004', '0004A30B00AC0000', '3C1F6A9E2B7D4E8F90A1B2C3D4E5F604'),
    ('sensor-bin-5', 'GED-2026-0005', '0004A30B00AC0005', '0004A30B00AC0000', '3C1F6A9E2B7D4E8F90A1B2C3D4E5F605'),
    ('sensor-bin-6', 'GED-2026-0006', '0004A30B00AC0006', '0004A30B00AC0000', '3C1F6A9E2B7D4E8F90A1B2C3D4E5F606'),
    ('sensor-bin-7', 'GED-2026-0007', '0004A30B00AC0007', '0004A30B00AC0000', '3C1F6A9E2B7D4E8F90A1B2C3D4E5F607');

INSERT INTO sensor_attachments (sensor_id, module_key, target_kind, target_id) VALUES
    ('sensor-bin-1', 'waste', 'bin', '019e3fcb-2a5f-7001-8000-000000000001'::uuid),
    ('sensor-bin-2', 'waste', 'bin', '019e3fcb-2a5f-7001-8000-000000000002'::uuid),
    ('sensor-bin-3', 'waste', 'bin', '019e3fcb-2a5f-7001-8000-000000000003'::uuid),
    ('sensor-bin-4', 'waste', 'bin', '019e3fcb-2a5f-7001-8000-000000000004'::uuid),
    ('sensor-bin-5', 'waste', 'bin', '019e3fcb-2a5f-7001-8000-000000000006'::uuid),
    ('sensor-bin-6', 'waste', 'bin', '019e3fcb-2a5f-7001-8000-000000000007'::uuid),
    ('sensor-bin-7', 'waste', 'bin', '019e3fcb-2a5f-7001-8000-000000000008'::uuid);

-- Three days of uplinks every three hours per sensor, the fill rising
-- linearly to its current share. The seed stamps the readings relative to
-- now, because a measurement older than sensor_offline_after (24 h by
-- default) counts as unknown; re-run the seeds when the demo looks stale.
-- Fill shares: B-001 0.92 and B-006 0.85 are above the 80 % threshold
-- (Voll), B-002 0.45 and B-008 0.60 are Normal, B-003 0.15 was measured
-- after today's tour emptied it (Normal), B-004 0.78 before (Geleert).
WITH uplinks AS (
    SELECT s.sensor_id, s.depth_cm, s.last_at - (n * interval '3 hours') AS at,
           round((s.depth_cm * (1 - s.final_fraction * (1 - n / 24.0)))::numeric, 1) AS distance_cm
    FROM (VALUES
        ('sensor-bin-1', 120, 0.92, now() - interval '35 minutes'),
        ('sensor-bin-2', 100, 0.45, now() - interval '50 minutes'),
        ('sensor-bin-3', 100, 0.15, now() - interval '20 minutes'),
        ('sensor-bin-4', 120, 0.78, now() - interval '3 hours'),
        ('sensor-bin-5', 120, 0.85, now() - interval '1 hour'),
        ('sensor-bin-7', 100, 0.60, now() - interval '40 minutes')
    ) AS s(sensor_id, depth_cm, final_fraction, last_at)
    CROSS JOIN generate_series(0, 23) AS n
),
readings AS (
    INSERT INTO sensor_data (id, sensor_id, updated_at, data)
    SELECT uuidv7_from_timestamp(at::timestamp), sensor_id, at::timestamp,
           jsonb_build_object('device', sensor_id, 'distance_cm', distance_cm, 'battery', 3.6)
    FROM uplinks
    RETURNING id, (data->>'distance_cm')::numeric AS distance_cm
)
INSERT INTO sensor_data_ability_values (sensor_data_id, sensor_model_ability_id, value)
SELECT r.id, sma.id, CASE a.ability WHEN 'distance' THEN r.distance_cm ELSE 3.6 END
FROM readings r
CROSS JOIN sensor_model_abilities sma
JOIN sensor_abilities a ON a.id = sma.sensor_ability_id
WHERE sma.sensor_model_id = (SELECT id FROM sensor_models WHERE name = 'GE Distance (simuliert)');

-- The fill level handler is not running during a seed, so derive the bins'
-- fill from their latest uplink the same way it would (1 - distance / depth).
UPDATE waste_bins b
   SET fill_fraction = LEAST(1, GREATEST(0, 1 - (latest.data->>'distance_cm')::float / b.depth_cm)),
       fill_measured_at = latest.updated_at
  FROM sensor_attachments sa
  JOIN LATERAL (
      SELECT d.data, d.updated_at
        FROM sensor_data d
       WHERE d.sensor_id = sa.sensor_id
       ORDER BY d.updated_at DESC
       LIMIT 1
  ) latest ON TRUE
 WHERE sa.module_key = 'waste' AND sa.target_kind = 'bin' AND sa.target_id = b.id;

-- Today's finished tour emptied B-003 and B-004 an hour ago.
UPDATE waste_bins
   SET last_emptied_at = now() - interval '1 hour'
 WHERE id IN ('019e3fcb-2a5f-7001-8000-000000000003'::uuid, '019e3fcb-2a5f-7001-8000-000000000004'::uuid);

-- One collection tour per board column: Geplant, Unterwegs, Erledigt.
-- Demand is volume × fill share (the whole volume for an unknown fill).
INSERT INTO operation_plans (id, module_key, date, description, status, distance, total_demand, cancellation_note, organization_id) VALUES
  ('019e3fcb-2a5e-7001-8000-000000000001'::uuid, 'waste', CURRENT_DATE + 2, 'Leerungstour Nord und Solitüde', 'planned', 18.6, 1340.0, '', '01980000-0000-7000-8000-000000000002'),
  ('019e3fcb-2a5e-7001-8000-000000000002'::uuid, 'waste', CURRENT_DATE, 'Leerungstour Innenstadt', 'active', 11.2, 2055.0, '', '01980000-0000-7000-8000-000000000002'),
  ('019e3fcb-2a5e-7001-8000-000000000003'::uuid, 'waste', CURRENT_DATE, 'Leerungstour Altstadt', 'finished', 7.4, 1050.0, '', '01980000-0000-7000-8000-000000000002');

INSERT INTO operation_plan_vehicles (vehicle_id, plan_id, role) VALUES
  ('019e3fcb-2a51-73cf-b612-cd0ccdac6f0d'::uuid, '019e3fcb-2a5e-7001-8000-000000000001'::uuid, 'transporter'),
  ('019e3fcb-2a51-7601-8000-000000000601'::uuid, '019e3fcb-2a5e-7001-8000-000000000002'::uuid, 'transporter'),
  ('019e3fcb-2a51-7601-8000-000000000601'::uuid, '019e3fcb-2a5e-7001-8000-000000000003'::uuid, 'transporter');

-- tbz.routen drives the collection tours as well.
INSERT INTO operation_plan_users (user_id, plan_id) VALUES
  ('01980000-0000-7000-8001-000000000014', '019e3fcb-2a5e-7001-8000-000000000001'::uuid),
  ('01980000-0000-7000-8001-000000000014', '019e3fcb-2a5e-7001-8000-000000000002'::uuid),
  ('01980000-0000-7000-8001-000000000014', '019e3fcb-2a5e-7001-8000-000000000003'::uuid);

INSERT INTO operation_plan_stops (stop_id, plan_id, completed, amount) VALUES
  ('019e3fcb-2a5f-7001-8000-000000000007'::uuid, '019e3fcb-2a5e-7001-8000-000000000001'::uuid, FALSE, NULL),
  ('019e3fcb-2a5f-7001-8000-000000000005'::uuid, '019e3fcb-2a5e-7001-8000-000000000001'::uuid, FALSE, NULL),
  ('019e3fcb-2a5f-7001-8000-000000000001'::uuid, '019e3fcb-2a5e-7001-8000-000000000002'::uuid, FALSE, NULL),
  ('019e3fcb-2a5f-7001-8000-000000000002'::uuid, '019e3fcb-2a5e-7001-8000-000000000002'::uuid, FALSE, NULL),
  ('019e3fcb-2a5f-7001-8000-000000000006'::uuid, '019e3fcb-2a5e-7001-8000-000000000002'::uuid, FALSE, NULL),
  ('019e3fcb-2a5f-7001-8000-000000000003'::uuid, '019e3fcb-2a5e-7001-8000-000000000003'::uuid, TRUE, 190.0),
  ('019e3fcb-2a5f-7001-8000-000000000004'::uuid, '019e3fcb-2a5e-7001-8000-000000000003'::uuid, TRUE, 860.0);
