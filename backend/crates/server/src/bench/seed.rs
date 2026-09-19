//! Large-data seeder for the benchmark database.
//!
//! Rows are generated server-side with `generate_series` rather than inserted
//! one by one from the client: half a million trees is the difference between
//! seconds and minutes. Every statement is a raw `sqlx::query`, never a
//! `query!` macro, so nothing here lands in the offline query cache.
//!
//! Seeding is a top-up, not a rebuild. Each function inserts only the
//! difference between the rows already present and the target scale, which is
//! what lets the campaign grow through the scales and measure after each step.

use sqlx::PgPool;

use crate::bench::scale::Scale;

/// Seeded by migration; every tenant hangs below it.
pub const ROOT_ORG_ID: &str = "01980000-0000-7000-8000-000000000001";

/// The benchmark user whose access context is measured. Fixed so the harness
/// can look it up without first querying for "some user".
pub const BENCH_USER_ID: &str = "01990000-0000-7000-8000-0000000000be";

/// Tenants the benchmark user holds roles in. A real user holds a handful of
/// grants however large the installation gets, and `visible_orgs` walks the
/// whole organization tree once per grant.
const GRANTED_TENANTS: i64 = 8;

/// Tenants directly under the root. A large installation is wide, not deep.
const TENANT_COUNT: i32 = 200;

/// One chain this deep hangs off the first tenant. The visibility filter's
/// ancestry walk costs depth, not width, so the flat shape alone would hide it.
const CHAIN_DEPTH: i32 = 8;

pub struct SeedPlan {
    pub scale: Scale,
    pub history_days: i64,
    pub seed: u64,
}

/// The sensor family the seeded devices use. GES-1000 reports volumetric soil
/// moisture at two depths, which is what the cluster and sensor moisture
/// series consume; a soil-tension model would leave those paths empty.
const SENSOR_MODEL: &str = "GES-1000";

/// Uplinks per sensor per day. LoRaWAN soil sensors report a handful of times
/// daily, not continuously.
const READINGS_PER_DAY: i64 = 4;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct SeedCounts {
    pub organizations: i64,
    pub clusters: i64,
    pub trees: i64,
    pub sensors: i64,
    pub readings: i64,
    pub plans: i64,
    pub grants: i64,
}

pub async fn seed_core(pool: &PgPool, plan: &SeedPlan) -> Result<SeedCounts, sqlx::Error> {
    install_uuid_v7(pool).await?;

    // Makes random() reproducible for this session, so two runs with the same
    // --seed produce the same column values. The ids differ regardless: they
    // carry a clock reading by construction.
    sqlx::query("SELECT setseed($1)")
        .bind((plan.seed % 1000) as f64 / 1000.0)
        .execute(pool)
        .await?;

    seed_organizations(pool).await?;
    let clusters = seed_clusters(pool, plan.scale).await?;
    let trees = seed_trees(pool, plan.scale).await?;
    let sensors = seed_sensors(pool, plan.scale).await?;
    let readings = seed_readings(pool, plan.history_days).await?;
    let plans = seed_plans(pool, plan.scale).await?;
    let grants = seed_user_and_roles(pool).await?;

    Ok(SeedCounts {
        organizations: count(pool, "organizations").await?,
        clusters,
        trees,
        sensors,
        readings,
        plans,
        grants,
    })
}

/// Org-owned role copies plus one user holding them, so the per-request
/// `AccessContext` can be measured at all. Creating an organization through
/// the service instantiates these copies; the seeder writes SQL directly and
/// therefore has to do it itself.
async fn seed_user_and_roles(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO roles (id, organization_id, name, description, permissions, template_key)
        SELECT bench_uuid_v7(), t.id, tpl.name, tpl.description, tpl.permissions, tpl.template_key
        FROM (
            SELECT id FROM organizations
            WHERE parent_id IS NOT NULL
            ORDER BY id
            LIMIT $1
        ) AS t
        CROSS JOIN (SELECT * FROM roles WHERE organization_id IS NULL) AS tpl
        ON CONFLICT (organization_id, name) DO NOTHING
        "#,
    )
    .bind(GRANTED_TENANTS)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO user_profiles (id, organization_id)
        SELECT $1::uuid, id
        FROM organizations
        WHERE parent_id IS NOT NULL
        ORDER BY id
        LIMIT 1
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(BENCH_USER_ID)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO role_assignments (user_id, role_id)
        SELECT $1::uuid, r.id
        FROM roles r
        WHERE r.organization_id IN (
            SELECT id FROM organizations
            WHERE parent_id IS NOT NULL
            ORDER BY id
            LIMIT $2
        )
        ON CONFLICT (user_id, role_id) DO NOTHING
        "#,
    )
    .bind(BENCH_USER_ID)
    .bind(GRANTED_TENANTS)
    .execute(pool)
    .await?;

    count(pool, "role_assignments").await
}

/// Postgres 17 has no v7 generator, and `gen_random_uuid` yields v4. That is
/// not a cosmetic difference: `pg_tree.rs` derives `TreeView::created_at` from
/// the id's embedded v7 timestamp and `expect`s it to be present, so v4 ids
/// would panic on every read. Random ids would also scatter primary-key inserts
/// across the B-tree, which production never does.
async fn install_uuid_v7(pool: &PgPool) -> Result<(), sqlx::Error> {
    // The timestamp is a parameter because a sensor reading's id *is* its
    // clock: `SensorReading::reconstitute` derives `recorded_at` from it and
    // `latest` orders by `id DESC`. Stamping insert time would collapse a
    // two-year time series onto a single moment.
    sqlx::query(
        r#"
        CREATE OR REPLACE FUNCTION bench_uuid_v7_at(ts timestamptz) RETURNS uuid
        LANGUAGE sql VOLATILE AS $fn$
            SELECT (
                lpad(to_hex((extract(epoch from ts) * 1000)::bigint), 12, '0')
                || '7' || substr(md5(random()::text), 1, 3)
                || (ARRAY['8','9','a','b'])[1 + floor(random() * 4)::int]
                || substr(md5(random()::text), 1, 3)
                || substr(md5(random()::text || ts::text), 1, 12)
            )::uuid
        $fn$
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE OR REPLACE FUNCTION bench_uuid_v7() RETURNS uuid
        LANGUAGE sql VOLATILE AS $fn$
            SELECT bench_uuid_v7_at(clock_timestamp())
        $fn$
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn seed_organizations(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO organizations (id, parent_id, name)
        SELECT bench_uuid_v7(), $1::uuid, 'Bench-Mandant ' || i
        FROM generate_series(1, $2) AS i
        ON CONFLICT (parent_id, name) DO NOTHING
        "#,
    )
    .bind(ROOT_ORG_ID)
    .bind(TENANT_COUNT)
    .execute(pool)
    .await?;

    for depth in 1..=CHAIN_DEPTH {
        sqlx::query(
            r#"
            INSERT INTO organizations (id, parent_id, name)
            SELECT bench_uuid_v7(), parent.id, 'Bench-Tiefe ' || $1
            FROM (
                SELECT id FROM organizations
                WHERE name = CASE WHEN $1 = 1 THEN 'Bench-Mandant 1'
                                  ELSE 'Bench-Tiefe ' || ($1 - 1) END
                LIMIT 1
            ) AS parent
            ON CONFLICT (parent_id, name) DO NOTHING
            "#,
        )
        .bind(depth)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn seed_clusters(pool: &PgPool, scale: Scale) -> Result<i64, sqlx::Error> {
    let existing = count(pool, "tree_clusters").await?;
    let missing = scale.clusters() - existing;
    if missing <= 0 {
        return Ok(existing);
    }

    // Every per-row value is computed inside a CTE over generate_series.
    // A scalar subquery or an uncorrelated LATERAL would be evaluated once for
    // the whole statement, giving every cluster the same coordinate and the
    // same owner, which reads fast and measures nothing.
    sqlx::query(
        r#"
        WITH tenants AS (
            SELECT id,
                   row_number() OVER (ORDER BY id) AS rn,
                   count(*) OVER ()                AS total
            FROM organizations
            WHERE parent_id IS NOT NULL
        ),
        generated AS (
            SELECT $2 + i               AS seq,
                   54.75 + random() * 0.1 AS lat,
                   9.40  + random() * 0.1 AS lng
            FROM generate_series(1, $1) AS i
        )
        INSERT INTO tree_clusters
            (id, region_id, name, address, description, moisture_level,
             soil_condition, watering_status, archived, latitude, longitude,
             geometry, organization_id)
        SELECT
            bench_uuid_v7(),
            NULL,
            'Bench-Cluster ' || g.seq,
            'Benchweg ' || g.seq || ', 24937 Flensburg',
            'seeded',
            0.5,
            'Lu'::tree_soil_condition,
            (ARRAY['good','moderate','bad','unknown'])[1 + (g.seq % 4)]::watering_status,
            false,
            g.lat,
            g.lng,
            ST_SetSRID(ST_MakePoint(g.lng, g.lat), 4326),
            t.id
        FROM generated g
        JOIN tenants t ON t.rn = 1 + (g.seq % t.total)
        "#,
    )
    .bind(missing)
    .bind(existing)
    .execute(pool)
    .await?;

    count(pool, "tree_clusters").await
}

/// Tree numbers deliberately mix a lettered prefix with a digit part: the
/// list's default sort splits `number` by regex and casts the digits to
/// numeric, and pure digits would exercise only half of that path. Coordinates
/// scatter around their cluster's centre rather than over the whole rectangle,
/// because a spatial index behaves differently on clustered points.
async fn seed_trees(pool: &PgPool, scale: Scale) -> Result<i64, sqlx::Error> {
    let existing = count(pool, "trees").await?;
    let missing = scale.trees() - existing;
    if missing <= 0 {
        return Ok(existing);
    }

    sqlx::query(
        r#"
        WITH clusters AS (
            SELECT id, organization_id, latitude, longitude,
                   row_number() OVER (ORDER BY id) AS rn,
                   count(*) OVER ()                AS total
            FROM tree_clusters
        )
        INSERT INTO trees
            (id, tree_cluster_id, sensor_id, number, species, planting_year,
             latitude, longitude, geometry, watering_status, last_watered,
             description, provider, organization_id)
        SELECT
            bench_uuid_v7(),
            c.id,
            NULL,
            CASE WHEN (n % 3) = 0 THEN '' ELSE chr(65 + (n % 26)::int) END || (1000 + n),
            (ARRAY['Acer platanoides','Tilia cordata','Quercus robur','Fagus sylvatica'])[1 + (n % 4)],
            1950 + (n % 70)::int,
            pos.lat,
            pos.lng,
            ST_SetSRID(ST_MakePoint(pos.lng, pos.lat), 4326),
            (ARRAY['good','moderate','bad','unknown'])[1 + (n % 4)]::watering_status,
            NULL,
            NULL,
            NULL,
            c.organization_id
        FROM generate_series($2 + 1, $2 + $1) AS n
        JOIN clusters c ON c.rn = 1 + (n % c.total)
        CROSS JOIN LATERAL (
            SELECT c.latitude  + (random() - 0.5) * 0.002 AS lat,
                   c.longitude + (random() - 0.5) * 0.002 AS lng
        ) AS pos
        "#,
    )
    .bind(missing)
    .bind(existing)
    .execute(pool)
    .await?;

    count(pool, "trees").await
}

/// EUIs are 16 hex characters, so the sequence number is zero-padded into that
/// shape. Every sensor is activated and linked to a tree, because a prepared
/// sensor carries no readings and would not exercise anything.
async fn seed_sensors(pool: &PgPool, scale: Scale) -> Result<i64, sqlx::Error> {
    let existing = count(pool, "sensors").await?;
    let missing = scale.sensors() - existing;
    if missing <= 0 {
        return Ok(existing);
    }

    sqlx::query(
        r#"
        WITH tenants AS (
            SELECT id,
                   row_number() OVER (ORDER BY id) AS rn,
                   count(*) OVER ()                AS total
            FROM organizations
            WHERE parent_id IS NOT NULL
        ),
        generated AS (
            SELECT $2 + i AS seq FROM generate_series(1, $1) AS i
        )
        INSERT INTO sensors (id, model_id, type, activated_at, organization_id)
        SELECT
            lpad(to_hex(g.seq), 16, '0'),
            (SELECT id FROM sensor_models WHERE name = $3),
            'lorawan'::sensor_type,
            now() - interval '1 year',
            t.id
        FROM generated g
        JOIN tenants t ON t.rn = 1 + (g.seq % t.total)
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(missing)
    .bind(existing)
    .bind(SENSOR_MODEL)
    .execute(pool)
    .await?;

    // The tree link is what production establishes on activation; the sensor
    // aggregate itself carries no position.
    sqlx::query(
        r#"
        WITH unlinked AS (
            SELECT s.id AS sensor_id, row_number() OVER (ORDER BY s.id) AS rn
            FROM sensors s
            WHERE NOT EXISTS (SELECT 1 FROM trees t WHERE t.sensor_id = s.id)
        ),
        targets AS (
            SELECT t.id AS tree_id, row_number() OVER (ORDER BY t.id) AS rn
            FROM trees t
            WHERE t.sensor_id IS NULL
        )
        UPDATE trees t
        SET sensor_id = unlinked.sensor_id
        FROM unlinked
        JOIN targets ON targets.rn = unlinked.rn
        WHERE t.id = targets.tree_id
        "#,
    )
    .execute(pool)
    .await?;

    // A sensor belongs to whoever owns the tree it sits on; production keeps
    // the two in step through the transfer flows.
    sqlx::query(
        r#"
        UPDATE sensors s
        SET organization_id = t.organization_id
        FROM trees t
        WHERE t.sensor_id = s.id
          AND s.organization_id <> t.organization_id
        "#,
    )
    .execute(pool)
    .await?;

    count(pool, "sensors").await
}

/// Readings are only seeded for sensors that have none yet, so growing into
/// the next scale extends the series sideways (more sensors) rather than
/// duplicating history for the existing ones.
async fn seed_readings(pool: &PgPool, history_days: i64) -> Result<i64, sqlx::Error> {
    sqlx::query(
        r#"
        WITH fresh AS (
            SELECT s.id
            FROM sensors s
            WHERE NOT EXISTS (
                SELECT 1 FROM sensor_data d WHERE d.sensor_id = s.id
            )
        ),
        stamps AS (
            SELECT f.id AS sensor_id,
                   now() - (d.day || ' days')::interval - (h.hour || ' hours')::interval AS at
            FROM fresh f
            CROSS JOIN generate_series(0, $1 - 1) AS d(day)
            CROSS JOIN generate_series(0, 23, 24 / $2::int) AS h(hour)
        )
        INSERT INTO sensor_data (id, sensor_id, updated_at, data)
        SELECT
            bench_uuid_v7_at(stamps.at),
            stamps.sensor_id,
            stamps.at,
            jsonb_build_object(
                'battery', round((3.2 + random() * 0.4)::numeric, 2),
                'temperature', round((8 + random() * 16)::numeric, 1),
                'soil_moisture', jsonb_build_array(
                    jsonb_build_object('depth_cm', 40, 'moisture_percent', round((15 + random() * 30)::numeric, 1)),
                    jsonb_build_object('depth_cm', 80, 'moisture_percent', round((15 + random() * 30)::numeric, 1))
                )
            )
        FROM stamps
        "#,
    )
    .bind(history_days)
    .bind(READINGS_PER_DAY)
    .execute(pool)
    .await?;

    // The normalized per-ability values are a separate table, and every reader
    // that asks for moisture by depth goes through it rather than the jsonb.
    sqlx::query(
        r#"
        INSERT INTO sensor_data_ability_values
            (sensor_data_id, sensor_model_ability_id, value, plausible)
        SELECT
            d.id,
            sma.id,
            CASE a.ability
                WHEN 'soil_moisture' THEN round((15 + random() * 30)::numeric, 1)
                WHEN 'temperature'   THEN round((8 + random() * 16)::numeric, 1)
                ELSE round((3.2 + random() * 0.4)::numeric, 2)
            END,
            true
        FROM sensor_data d
        JOIN sensors s ON s.id = d.sensor_id
        JOIN sensor_model_abilities sma ON sma.sensor_model_id = s.model_id
        JOIN sensor_abilities a ON a.id = sma.sensor_ability_id
        WHERE NOT EXISTS (
            SELECT 1 FROM sensor_data_ability_values existing
            WHERE existing.sensor_data_id = d.id
        )
        "#,
    )
    .execute(pool)
    .await?;

    count(pool, "sensor_data").await
}

async fn seed_plans(pool: &PgPool, scale: Scale) -> Result<i64, sqlx::Error> {
    let existing = count(pool, "watering_plans").await?;
    let missing = scale.plans() - existing;
    if missing <= 0 {
        return Ok(existing);
    }

    sqlx::query(
        r#"
        WITH tenants AS (
            SELECT id,
                   row_number() OVER (ORDER BY id) AS rn,
                   count(*) OVER ()                AS total
            FROM organizations
            WHERE parent_id IS NOT NULL
        )
        INSERT INTO vehicles
            (id, number_plate, model, description, type, availability,
             driving_license, water_capacity, width, height, length, weight,
             organization_id)
        SELECT
            bench_uuid_v7(),
            'FL-BE ' || i,
            'Bench-Modell',
            'seeded',
            'transporter'::vehicle_type,
            'available'::vehicle_availability,
            'B'::driving_license,
            2000, 2.2, 2.5, 6.0, 3500,
            t.id
        FROM generate_series(1, 20) AS i
        JOIN tenants t ON t.rn = 1 + (i % t.total)
        ON CONFLICT DO NOTHING
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        WITH tenants AS (
            SELECT id,
                   row_number() OVER (ORDER BY id) AS rn,
                   count(*) OVER ()                AS total
            FROM organizations
            WHERE parent_id IS NOT NULL
        )
        INSERT INTO watering_plans
            (id, date, status, description, distance, total_water_required,
             duration, refill_count, organization_id)
        SELECT
            bench_uuid_v7(),
            (current_date - ((($2 + i) % 730)::int))::date,
            'finished'::watering_plan_status,
            'seeded',
            12000 + random() * 8000,
            600 + random() * 400,
            4 + random() * 4,
            1,
            t.id
        FROM generate_series(1, $1) AS i
        JOIN tenants t ON t.rn = 1 + (($2 + i) % t.total)
        "#,
    )
    .bind(missing)
    .bind(existing)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO tree_cluster_watering_plans (tree_cluster_id, watering_plan_id)
        SELECT c.id, p.id
        FROM watering_plans p
        CROSS JOIN LATERAL (
            SELECT id FROM tree_clusters ORDER BY random() LIMIT 3
        ) AS c
        WHERE NOT EXISTS (
            SELECT 1 FROM tree_cluster_watering_plans existing
            WHERE existing.watering_plan_id = p.id
        )
        ON CONFLICT DO NOTHING
        "#,
    )
    .execute(pool)
    .await?;

    count(pool, "watering_plans").await
}

async fn count(pool: &PgPool, table: &str) -> Result<i64, sqlx::Error> {
    // The table name comes from this module's own string literals, never from
    // input, so the format! cannot carry caller text into SQL.
    sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
        .fetch_one(pool)
        .await
}

/// Without fresh statistics the planner keeps choosing the plan for the
/// previous table size, and the next measurement is an artefact of that.
pub async fn analyze(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "ANALYZE organizations, roles, role_assignments, user_profiles, tree_clusters, trees, sensors, sensor_data, sensor_data_ability_values, vehicles, watering_plans",
    )
        .execute(pool)
        .await?;
    Ok(())
}
