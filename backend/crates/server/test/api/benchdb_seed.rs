use domain::{
    sensor::repository::SensorReadingReader,
    shared::pagination::Pagination,
    tree::{TreeSearchQuery, repository::TreeReader},
};
use server::{
    bench::{
        scale::Scale,
        seed::{SeedPlan, seed_core},
    },
    infra::pg_tree::PgTreeRepository,
};

use crate::helpers::spawn_app;

fn xs_plan() -> SeedPlan {
    SeedPlan {
        scale: Scale::Xs,
        history_days: 2,
        seed: 42,
    }
}

#[tokio::test]
async fn seeding_xs_produces_rows_the_readers_can_consume() {
    let app = spawn_app().await;

    let counts = seed_core(&app.db_pool, &xs_plan())
        .await
        .expect("seeding must succeed against a migrated schema");

    assert_eq!(counts.trees, 1_000);
    assert_eq!(counts.clusters, 20);

    let repo = PgTreeRepository::new(app.db_pool.clone());
    let page = repo
        .view_search(TreeSearchQuery::default(), Pagination::new(1, 25))
        .await
        .expect("the seeded rows must be readable through the repository");

    assert_eq!(page.page.items.len(), 25);
    assert!(page.page.total >= 1_000, "got {}", page.page.total);
}

#[tokio::test]
async fn seeded_tree_ids_are_uuid_v7() {
    let app = spawn_app().await;

    seed_core(&app.db_pool, &xs_plan())
        .await
        .expect("seeding must succeed");

    // pg_tree.rs derives TreeView::created_at from the id's embedded v7
    // timestamp and `expect`s it to be there. A v4 id would panic every read.
    let non_v7: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM trees WHERE uuid_extract_version(id) IS DISTINCT FROM 7",
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("the version probe must run");

    assert_eq!(non_v7, 0, "every seeded tree id must be a uuid v7");
}

#[tokio::test]
async fn seeded_tree_numbers_exercise_the_default_sort() {
    let app = spawn_app().await;

    seed_core(&app.db_pool, &xs_plan())
        .await
        .expect("seeding must succeed");

    // The list's default sort splits `number` into a letter prefix and a digit
    // part by regex. Numbers of only one shape would exercise half the path.
    let with_prefix: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM trees WHERE number ~ '^[A-Z]'")
        .fetch_one(&app.db_pool)
        .await
        .expect("the prefix probe must run");
    let without_prefix: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM trees WHERE number ~ '^[0-9]'")
            .fetch_one(&app.db_pool)
            .await
            .expect("the digit probe must run");

    assert!(with_prefix > 0, "no lettered tree numbers were seeded");
    assert!(
        without_prefix > 0,
        "no bare numeric tree numbers were seeded"
    );
}

#[tokio::test]
async fn seeding_twice_tops_up_instead_of_duplicating() {
    let app = spawn_app().await;

    seed_core(&app.db_pool, &xs_plan())
        .await
        .expect("first seed must succeed");
    let second = seed_core(&app.db_pool, &xs_plan())
        .await
        .expect("second seed must succeed");

    // The campaign grows through the scales by re-running the seeder, so a
    // repeated call at the same scale has to be a no-op rather than a doubling.
    assert_eq!(second.trees, 1_000);
    assert_eq!(second.clusters, 20);
}

#[tokio::test]
async fn seeded_readings_are_readable_and_spread_over_time() {
    let app = spawn_app().await;

    let counts = seed_core(&app.db_pool, &xs_plan())
        .await
        .expect("seeding must succeed");

    assert_eq!(counts.sensors, 10);
    // 10 sensors * 4 uplinks per day * 2 days
    assert_eq!(counts.readings, 80);

    let sensor_id: String = sqlx::query_scalar("SELECT id FROM sensors LIMIT 1")
        .fetch_one(&app.db_pool)
        .await
        .expect("a seeded sensor must exist");
    let sensor_id = domain::sensor::SensorId::new(&sensor_id).expect("seeded EUI must be valid");

    let repo = server::infra::pg_sensor::PgSensorRepository::new(
        app.db_pool.clone(),
        chrono::Duration::days(1),
        3,
    );

    let latest = repo
        .latest(&sensor_id)
        .await
        .expect("the reader must accept the seeded readings");
    let latest = latest.expect("the seeded time series must be readable");

    let history = repo
        .history(&sensor_id, 100)
        .await
        .expect("history must read back");
    assert_eq!(history.len(), 8, "4 uplinks per day over 2 days");

    // recorded_at is derived from the id's v7 timestamp, so ids carrying insert
    // time would collapse the whole series onto one moment.
    let oldest = history
        .iter()
        .map(|r| r.recorded_at)
        .min()
        .expect("history is not empty");
    assert!(
        latest.recorded_at - oldest > chrono::Duration::hours(24),
        "expected the series to span more than a day, got {} to {}",
        oldest,
        latest.recorded_at
    );

    let moisture = repo
        .latest_volumetric_moisture(&sensor_id)
        .await
        .expect("the normalized ability values must read back");
    assert_eq!(
        moisture.len(),
        2,
        "GES-1000 reports moisture at 40 and 80 cm"
    );
}

#[tokio::test]
async fn seeded_rows_are_spread_rather_than_identical() {
    let app = spawn_app().await;

    let counts = seed_core(&app.db_pool, &xs_plan())
        .await
        .expect("seeding must succeed");

    // An uncorrelated scalar subquery is evaluated once for the whole
    // statement, which silently gives every row the same value. That produces
    // a database which seeds fast, reads fast and measures nothing: all points
    // land on one spot, so no spatial index is exercised, and all rows belong
    // to one organization, so no scope filter is either.
    let distinct_coords: i64 =
        sqlx::query_scalar("SELECT COUNT(DISTINCT (latitude, longitude)) FROM tree_clusters")
            .fetch_one(&app.db_pool)
            .await
            .expect("the coordinate probe must run");
    assert_eq!(
        distinct_coords, counts.clusters,
        "every cluster must sit somewhere of its own"
    );

    let distinct_cluster_orgs: i64 =
        sqlx::query_scalar("SELECT COUNT(DISTINCT organization_id) FROM tree_clusters")
            .fetch_one(&app.db_pool)
            .await
            .expect("the cluster org probe must run");
    assert!(
        distinct_cluster_orgs > 1,
        "clusters must span several organizations, got {distinct_cluster_orgs}"
    );

    let distinct_tree_orgs: i64 =
        sqlx::query_scalar("SELECT COUNT(DISTINCT organization_id) FROM trees")
            .fetch_one(&app.db_pool)
            .await
            .expect("the tree org probe must run");
    assert!(
        distinct_tree_orgs > 1,
        "trees must span several organizations, got {distinct_tree_orgs}"
    );

    // Roughly 11 km by 6 km of scatter; a degenerate seed collapses to metres.
    let span: f64 = sqlx::query_scalar(
        "SELECT ST_Distance(
             ST_SetSRID(ST_MakePoint(MIN(longitude), MIN(latitude)), 4326)::geography,
             ST_SetSRID(ST_MakePoint(MAX(longitude), MAX(latitude)), 4326)::geography
         ) FROM trees",
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("the span probe must run");
    assert!(
        span > 5_000.0,
        "expected trees to span kilometres, got {span:.0} m"
    );
}

#[tokio::test]
async fn seeding_creates_a_user_with_role_grants() {
    let app = spawn_app().await;

    let counts = seed_core(&app.db_pool, &xs_plan())
        .await
        .expect("seeding must succeed");

    // Without org-owned roles and a user holding them, AccessContext
    // construction has nothing to resolve and the measurement is vacuous.
    assert!(counts.grants > 0, "no role assignments were seeded");

    let org_roles: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM roles WHERE organization_id IS NOT NULL")
            .fetch_one(&app.db_pool)
            .await
            .expect("the role probe must run");
    assert!(org_roles > 0, "no organization-owned roles were seeded");

    let service = server::service::authorization::AuthorizationService::new(
        std::sync::Arc::new(
            server::infra::pg_organization::PgOrganizationRepository::new(app.db_pool.clone()),
        ),
        std::sync::Arc::new(server::infra::pg_role::PgRoleRepository::new(
            app.db_pool.clone(),
        )),
        true,
    );
    let user_id: uuid::Uuid = server::bench::seed::BENCH_USER_ID
        .parse()
        .expect("the constant is a valid uuid");

    let context = service
        .context_for(user_id)
        .await
        .expect("the access context must resolve");

    assert!(
        !context.permissions.grants().is_empty(),
        "the seeded user must actually hold grants"
    );
    assert!(
        !context.permissions.is_unrestricted(),
        "an unrestricted context would mean the enforcement bypass kicked in"
    );
}

#[tokio::test]
async fn seeded_reference_data_makes_the_dashboard_paths_non_empty() {
    let app = spawn_app().await;

    seed_core(&app.db_pool, &xs_plan())
        .await
        .expect("seeding must succeed");

    // A path that returns nothing looks fast and says nothing. Regions, the
    // vehicle-to-plan link and a share of implausible readings are what keep
    // the dashboard and quality paths from measuring an empty result.
    for (label, sql) in [
        (
            "clusters assigned to a region",
            "SELECT COUNT(*) FROM tree_clusters WHERE region_id IS NOT NULL",
        ),
        (
            "vehicles linked to a plan",
            "SELECT COUNT(*) FROM vehicle_watering_plans",
        ),
        (
            "implausible ability values",
            "SELECT COUNT(*) FROM sensor_data_ability_values WHERE NOT plausible",
        ),
    ] {
        let count: i64 = sqlx::query_scalar(sql)
            .fetch_one(&app.db_pool)
            .await
            .unwrap_or_else(|e| panic!("the probe for {label} must run: {e}"));
        assert!(count > 0, "expected seeded {label}, got {count}");
    }
}
