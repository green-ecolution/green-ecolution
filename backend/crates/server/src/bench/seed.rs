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

#[derive(Debug, Default, PartialEq, Eq)]
pub struct SeedCounts {
    pub organizations: i64,
    pub clusters: i64,
    pub trees: i64,
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

    Ok(SeedCounts {
        organizations: count(pool, "organizations").await?,
        clusters,
        trees,
    })
}

/// Postgres 17 has no v7 generator, and `gen_random_uuid` yields v4. That is
/// not a cosmetic difference: `pg_tree.rs` derives `TreeView::created_at` from
/// the id's embedded v7 timestamp and `expect`s it to be present, so v4 ids
/// would panic on every read. Random ids would also scatter primary-key inserts
/// across the B-tree, which production never does.
async fn install_uuid_v7(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE OR REPLACE FUNCTION bench_uuid_v7() RETURNS uuid
        LANGUAGE sql VOLATILE AS $fn$
            SELECT (
                lpad(to_hex((extract(epoch from clock_timestamp()) * 1000)::bigint), 12, '0')
                || '7' || substr(md5(random()::text), 1, 3)
                || (ARRAY['8','9','a','b'])[1 + floor(random() * 4)::int]
                || substr(md5(random()::text), 1, 3)
                || substr(md5(random()::text || clock_timestamp()::text), 1, 12)
            )::uuid
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

    sqlx::query(
        r#"
        INSERT INTO tree_clusters
            (id, region_id, name, address, description, moisture_level,
             soil_condition, watering_status, archived, latitude, longitude,
             geometry, organization_id)
        SELECT
            bench_uuid_v7(),
            NULL,
            'Bench-Cluster ' || ($2 + i),
            'Benchweg ' || ($2 + i) || ', 24937 Flensburg',
            'seeded',
            0.5,
            'Lu'::tree_soil_condition,
            (ARRAY['good','moderate','bad','unknown'])[1 + (i % 4)]::watering_status,
            false,
            lat.value,
            lng.value,
            ST_SetSRID(ST_MakePoint(lng.value, lat.value), 4326),
            (SELECT id FROM organizations WHERE parent_id IS NOT NULL
             OFFSET floor(random() * GREATEST((SELECT COUNT(*) FROM organizations WHERE parent_id IS NOT NULL), 1))
             LIMIT 1)
        FROM generate_series(1, $1) AS i
        CROSS JOIN LATERAL (SELECT 54.75 + random() * 0.1 AS value) AS lat
        CROSS JOIN LATERAL (SELECT 9.40 + random() * 0.1 AS value) AS lng
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
    sqlx::query("ANALYZE organizations, tree_clusters, trees")
        .execute(pool)
        .await?;
    Ok(())
}
