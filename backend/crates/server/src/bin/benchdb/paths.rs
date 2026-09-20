//! The measured read paths.
//!
//! Every path goes through the real repository implementation rather than
//! hand-written SQL, so a measurement covers the same code the HTTP layer
//! calls, including any N+1 pattern a service happens to have.

use std::{sync::Arc, time::Instant};

use domain::{
    Id,
    authorization::Visibility,
    cluster::{
        SoilMoistureBucket, TreeCluster, TreeClusterSearchQuery, repository::TreeClusterReader,
    },
    evaluation::EvaluationRepository,
    sensor::{
        SensorId, SensorSearchQuery,
        data::SensorReadingDraft,
        repository::{SensorReader, SensorReadingReader, SensorReadingWriter},
    },
    shared::{
        coordinates::Coordinate, distance::Distance, pagination::Pagination,
        watering_status::WateringStatus,
    },
    tree::{
        Tree, TreeSearchQuery,
        repository::TreeReader,
        sort::{TreeSort, TreeSortField},
    },
};
use server::{
    bench::{scale::Scale, seed::BENCH_USER_ID},
    infra::{
        pg_cluster::PgTreeClusterRepository, pg_evaluation::PgEvaluationRepository,
        pg_organization::PgOrganizationRepository, pg_role::PgRoleRepository,
        pg_sensor::PgSensorRepository, pg_tree::PgTreeRepository,
    },
    service::authorization::AuthorizationService,
};
use sqlx::PgPool;

use crate::measure::{ITERATIONS, Sample, WARMUP, summarize};

type Failure = Box<dyn std::error::Error>;

pub const NAMES: &[&str] = &[
    "tree.view_search.default_sort",
    "tree.view_search.status_sort",
    "tree.view_search.status_filter",
    "tree.view_search.free_text",
    "tree.view_search.cluster_filter",
    "tree.view_search.deep_offset",
    "tree.view_markers",
    "tree.view_nearest",
    "tree.find_nearest",
    "tree.view_by_ids",
    "tree.distinct_planting_years",
    "cluster.view_search",
    "cluster.view_markers",
    "cluster.boundaries",
    "cluster.statistics",
    "cluster.center_point",
    "cluster.soil_moisture_series",
    "cluster.watering_events",
    "sensor.view_search",
    "sensor.latest",
    "sensor.history",
    "sensor.view_history",
    "sensor.last_plausible_values",
    "sensor.quality_issues",
    "sensor.latest_volumetric_moisture",
    "sensor.soil_moisture_series",
    "sensor.record",
    "evaluation.regions_with_watering_plan",
    "evaluation.vehicle_with_watering_plan",
    "evaluation.total_consumed_water",
    "evaluation.watering_plan_user",
    "authorization.context_for",
];

/// Plan probes: shapes copied from `view_search` in
/// `crates/server/src/infra/pg_tree.rs` so `EXPLAIN` has something to plan.
/// Measurements always go through the repository; these copies are never the
/// source of truth and have to be refreshed when that query changes.
///
/// The filters are inlined as literals rather than parameters on purpose. A
/// `$1::text IS NULL` branch folds away at planning time, and the plan then
/// shows an index-only scan that no real request ever gets.
pub const TREE_COUNT_SQL: &str = r#"
SELECT
  COUNT(*) FILTER (
    WHERE number ILIKE '%Tilia%' ESCAPE '\' OR species ILIKE '%Tilia%' ESCAPE '\'
  ) AS total,
  COUNT(*) AS total_unfiltered
FROM trees
"#;

/// The row half, in the list's default order: prefix first, then the digits
/// numerically. No index can serve that, so the plan shows what the sort
/// actually costs.
pub const TREE_ROWS_SQL: &str = r#"
SELECT t.id, t.number, c.name
FROM trees t
LEFT JOIN tree_clusters c ON c.id = t.tree_cluster_id
ORDER BY
  substring(t.number from '^\D*') ASC NULLS LAST,
  NULLIF(substring(t.number from '\d+'), '')::numeric ASC NULLS LAST,
  t.id ASC
LIMIT 25 OFFSET 0
"#;

/// Runs the closure `WARMUP` times untimed, then `ITERATIONS` times timed.
/// The closure returns the row count so the sample records what the path
/// actually produced, not just how long it took.
async fn measure<F, Fut>(
    label: &str,
    scale: Scale,
    visibility_label: &str,
    mut run: F,
) -> Result<Sample, Failure>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<u64, Failure>>,
{
    for _ in 0..WARMUP {
        run().await?;
    }

    let mut durations = Vec::with_capacity(ITERATIONS);
    let mut rows = 0;
    for _ in 0..ITERATIONS {
        let started = Instant::now();
        rows = run().await?;
        durations.push(started.elapsed().as_secs_f64() * 1000.0);
    }

    Ok(summarize(
        label,
        scale.name(),
        visibility_label,
        durations,
        rows,
    ))
}

fn visibility_label(visibility: &Visibility) -> &'static str {
    match visibility {
        Visibility::Unrestricted => "unrestricted",
        Visibility::Only(_) => "scoped",
    }
}

/// A mid-sized subtree rather than a single organization: the scoped case only
/// says something once `organization_id = ANY(...)` carries more than one id.
async fn scoped_visibility(pool: &PgPool) -> Result<Visibility, Failure> {
    let ids: Vec<uuid::Uuid> = sqlx::query_scalar(
        "SELECT id FROM organizations WHERE parent_id IS NOT NULL ORDER BY id LIMIT 20",
    )
    .fetch_all(pool)
    .await?;
    Ok(Visibility::Only(ids.into_iter().map(Id::new).collect()))
}

async fn some_cluster_ids(pool: &PgPool, limit: i64) -> Result<Vec<uuid::Uuid>, Failure> {
    Ok(
        sqlx::query_scalar("SELECT id FROM tree_clusters ORDER BY id LIMIT $1")
            .bind(limit)
            .fetch_all(pool)
            .await?,
    )
}

async fn some_sensor_id(pool: &PgPool) -> Result<Option<SensorId>, Failure> {
    let raw: Option<String> = sqlx::query_scalar("SELECT id FROM sensors ORDER BY id LIMIT 1")
        .fetch_optional(pool)
        .await?;
    Ok(raw.and_then(|id| SensorId::new(&id).ok()))
}

/// A cluster that actually has trees with sensors on them, so the moisture
/// series has something to aggregate instead of returning an empty result.
async fn cluster_with_sensors(pool: &PgPool) -> Result<Option<Id<TreeCluster>>, Failure> {
    let id: Option<uuid::Uuid> = sqlx::query_scalar(
        r#"SELECT t.tree_cluster_id
           FROM trees t
           WHERE t.sensor_id IS NOT NULL AND t.tree_cluster_id IS NOT NULL
           LIMIT 1"#,
    )
    .fetch_optional(pool)
    .await?;
    Ok(id.map(Id::new))
}

async fn some_tree_ids(pool: &PgPool, limit: i64) -> Result<Vec<Id<Tree>>, Failure> {
    let ids: Vec<uuid::Uuid> = sqlx::query_scalar("SELECT id FROM trees ORDER BY id LIMIT $1")
        .bind(limit)
        .fetch_all(pool)
        .await?;
    Ok(ids.into_iter().map(Id::new).collect())
}

pub async fn measure_all(
    pool: &PgPool,
    scale: Scale,
    only: Option<&[String]>,
) -> Result<Vec<Sample>, Failure> {
    let wanted = |name: &str| only.is_none_or(|list| list.iter().any(|n| n == name));

    let trees = PgTreeRepository::new(pool.clone());
    let clusters = PgTreeClusterRepository::new(pool.clone());
    let sensors = PgSensorRepository::new(pool.clone(), chrono::Duration::days(1), 3);
    let evaluation = PgEvaluationRepository::new(pool.clone());
    // `enforced = true`, otherwise context_for short-circuits to unrestricted
    // and measures nothing.
    let authorization = AuthorizationService::new(
        Arc::new(PgOrganizationRepository::new(pool.clone())),
        Arc::new(PgRoleRepository::new(pool.clone())),
        true,
    );

    let first_page = Pagination::new(1, 25);
    let cluster_ids = some_cluster_ids(pool, 50).await?;
    let sensor_id = some_sensor_id(pool).await?;
    let centre = Coordinate::new(54.7937, 9.4469).expect("Flensburg is a valid coordinate");
    let radius = Distance::new(500.0).expect("500 m is a valid distance");

    let mut samples = Vec::new();

    for visibility in [Visibility::Unrestricted, scoped_visibility(pool).await?] {
        let label = visibility_label(&visibility);

        let base = |visible: Visibility| TreeSearchQuery {
            visible,
            ..TreeSearchQuery::default()
        };

        if wanted("tree.view_search.default_sort") {
            let query = base(visibility.clone());
            samples.push(
                measure("tree.view_search.default_sort", scale, label, || async {
                    let page = trees.view_search(query.clone(), first_page).await?;
                    Ok(page.page.items.len() as u64)
                })
                .await?,
            );
        }

        if wanted("tree.view_search.status_sort") {
            let query = TreeSearchQuery {
                sort: TreeSort {
                    field: TreeSortField::Status,
                    ..TreeSort::default()
                },
                ..base(visibility.clone())
            };
            samples.push(
                measure("tree.view_search.status_sort", scale, label, || async {
                    let page = trees.view_search(query.clone(), first_page).await?;
                    Ok(page.page.items.len() as u64)
                })
                .await?,
            );
        }

        if wanted("tree.view_search.status_filter") {
            let query = TreeSearchQuery {
                watering_statuses: vec![WateringStatus::Bad],
                ..base(visibility.clone())
            };
            samples.push(
                measure("tree.view_search.status_filter", scale, label, || async {
                    let page = trees.view_search(query.clone(), first_page).await?;
                    Ok(page.page.items.len() as u64)
                })
                .await?,
            );
        }

        if wanted("tree.view_search.free_text") {
            let query = TreeSearchQuery {
                q: Some("Tilia".to_string()),
                ..base(visibility.clone())
            };
            samples.push(
                measure("tree.view_search.free_text", scale, label, || async {
                    let page = trees.view_search(query.clone(), first_page).await?;
                    Ok(page.page.items.len() as u64)
                })
                .await?,
            );
        }

        if wanted("tree.view_search.cluster_filter") {
            let query = TreeSearchQuery {
                cluster_ids: cluster_ids.iter().copied().map(Id::new).collect(),
                ..base(visibility.clone())
            };
            samples.push(
                measure("tree.view_search.cluster_filter", scale, label, || async {
                    let page = trees.view_search(query.clone(), first_page).await?;
                    Ok(page.page.items.len() as u64)
                })
                .await?,
            );
        }

        if wanted("tree.view_search.deep_offset") {
            // The last page the data set has: OFFSET cost grows with the page
            // number, so the first page alone would hide it entirely.
            let last_page = (scale.trees() as u64 / 25).max(1);
            let deep = Pagination::new(last_page, 25);
            let query = base(visibility.clone());
            samples.push(
                measure("tree.view_search.deep_offset", scale, label, || async {
                    let page = trees.view_search(query.clone(), deep).await?;
                    Ok(page.page.items.len() as u64)
                })
                .await?,
            );
        }

        if wanted("tree.view_markers") {
            let query = base(visibility.clone());
            samples.push(
                measure("tree.view_markers", scale, label, || async {
                    let markers = trees.view_markers(query.clone()).await?;
                    Ok(markers.len() as u64)
                })
                .await?,
            );
        }

        if wanted("tree.view_nearest") {
            let visible = visibility.clone();
            samples.push(
                measure("tree.view_nearest", scale, label, || async {
                    let found = trees
                        .view_nearest(centre, radius, 50, visible.clone())
                        .await?;
                    Ok(found.len() as u64)
                })
                .await?,
            );
        }

        if wanted("cluster.view_search") {
            let query = TreeClusterSearchQuery {
                visible: visibility.clone(),
                ..TreeClusterSearchQuery::default()
            };
            samples.push(
                measure("cluster.view_search", scale, label, || async {
                    let page = clusters.view_search(query.clone(), first_page).await?;
                    Ok(page.page.items.len() as u64)
                })
                .await?,
            );
        }

        if wanted("cluster.view_markers") {
            let visible = visibility.clone();
            samples.push(
                measure("cluster.view_markers", scale, label, || async {
                    let markers = clusters.view_markers(visible.clone()).await?;
                    Ok(markers.len() as u64)
                })
                .await?,
            );
        }

        if wanted("cluster.boundaries") {
            let visible = visibility.clone();
            samples.push(
                measure("cluster.boundaries", scale, label, || async {
                    let boundaries = clusters.boundaries(visible.clone()).await?;
                    Ok(boundaries.len() as u64)
                })
                .await?,
            );
        }

        if wanted("cluster.statistics") {
            let visible = visibility.clone();
            samples.push(
                measure("cluster.statistics", scale, label, || async {
                    clusters.statistics(visible.clone()).await?;
                    Ok(1)
                })
                .await?,
            );
        }

        if wanted("tree.distinct_planting_years") {
            let visible = visibility.clone();
            samples.push(
                measure("tree.distinct_planting_years", scale, label, || async {
                    let years = trees.distinct_planting_years(visible.clone()).await?;
                    Ok(years.len() as u64)
                })
                .await?,
            );
        }

        if wanted("sensor.view_search") {
            let query = SensorSearchQuery {
                visible: visibility.clone(),
                ..SensorSearchQuery::default()
            };
            samples.push(
                measure("sensor.view_search", scale, label, || async {
                    let page = sensors.view_search(query.clone(), first_page).await?;
                    Ok(page.page.items.len() as u64)
                })
                .await?,
            );
        }

        if wanted("evaluation.regions_with_watering_plan") {
            let visible = visibility.clone();
            samples.push(
                measure(
                    "evaluation.regions_with_watering_plan",
                    scale,
                    label,
                    || async {
                        let rows = evaluation
                            .regions_with_watering_plan(visible.clone())
                            .await?;
                        Ok(rows.len() as u64)
                    },
                )
                .await?,
            );
        }

        if wanted("evaluation.vehicle_with_watering_plan") {
            let visible = visibility.clone();
            samples.push(
                measure(
                    "evaluation.vehicle_with_watering_plan",
                    scale,
                    label,
                    || async {
                        let rows = evaluation
                            .vehicle_with_watering_plan(visible.clone(), visible.clone())
                            .await?;
                        Ok(rows.len() as u64)
                    },
                )
                .await?,
            );
        }

        if wanted("evaluation.total_consumed_water") {
            let visible = visibility.clone();
            samples.push(
                measure("evaluation.total_consumed_water", scale, label, || async {
                    evaluation.total_consumed_water(visible.clone()).await?;
                    Ok(1)
                })
                .await?,
            );
        }

        if wanted("evaluation.watering_plan_user") {
            let visible = visibility.clone();
            samples.push(
                measure("evaluation.watering_plan_user", scale, label, || async {
                    evaluation.watering_plan_user(visible.clone()).await?;
                    Ok(1)
                })
                .await?,
            );
        }
    }

    // The remaining paths take no visibility argument, so measuring them once
    // per variant would record the same number twice.

    if wanted("tree.find_nearest") {
        samples.push(
            measure("tree.find_nearest", scale, "unrestricted", || async {
                let found = trees.find_nearest(centre, radius).await?;
                Ok(found.is_some() as u64)
            })
            .await?,
        );
    }

    if wanted("tree.view_by_ids") {
        // 500 ids is what a map viewport or a bulk edit hands over; the
        // interesting question is whether the ANY($1) lookup stays indexed.
        let ids = some_tree_ids(pool, 500).await?;
        samples.push(
            measure("tree.view_by_ids", scale, "unrestricted", || async {
                let views = trees.view_by_ids(&ids).await?;
                Ok(views.len() as u64)
            })
            .await?,
        );
    }

    if let Some(cluster_id) = cluster_with_sensors(pool).await? {
        if wanted("cluster.center_point") {
            samples.push(
                measure("cluster.center_point", scale, "unrestricted", || async {
                    let point = clusters.center_point(cluster_id).await?;
                    Ok(point.is_some() as u64)
                })
                .await?,
            );
        }

        if wanted("cluster.soil_moisture_series") {
            let to = chrono::Utc::now();
            let from = to - chrono::Duration::days(30);
            samples.push(
                measure(
                    "cluster.soil_moisture_series",
                    scale,
                    "unrestricted",
                    || async {
                        let series = clusters
                            .soil_moisture_series(cluster_id, from, to, SoilMoistureBucket::Day)
                            .await?;
                        Ok(series.len() as u64)
                    },
                )
                .await?,
            );
        }

        if wanted("cluster.watering_events") {
            samples.push(
                measure("cluster.watering_events", scale, "unrestricted", || async {
                    let events = clusters.watering_events(cluster_id).await?;
                    Ok(events.len() as u64)
                })
                .await?,
            );
        }
    }

    if wanted("authorization.context_for") {
        // Runs once per authenticated request before any query does, so its
        // cost is added to every other number in this table.
        let user_id: uuid::Uuid = BENCH_USER_ID.parse()?;
        samples.push(
            measure(
                "authorization.context_for",
                scale,
                "unrestricted",
                || async {
                    let context = authorization.context_for(user_id).await?;
                    Ok(context.permissions.grants().len() as u64)
                },
            )
            .await?,
        );
    }

    if let Some(sensor_id) = sensor_id {
        if wanted("sensor.latest") {
            samples.push(
                measure("sensor.latest", scale, "unrestricted", || async {
                    let latest = sensors.latest(&sensor_id).await?;
                    Ok(latest.is_some() as u64)
                })
                .await?,
            );
        }

        if wanted("sensor.history") {
            samples.push(
                measure("sensor.history", scale, "unrestricted", || async {
                    let history = sensors.history(&sensor_id, 200).await?;
                    Ok(history.len() as u64)
                })
                .await?,
            );
        }

        if wanted("sensor.latest_volumetric_moisture") {
            samples.push(
                measure(
                    "sensor.latest_volumetric_moisture",
                    scale,
                    "unrestricted",
                    || async {
                        let values = sensors.latest_volumetric_moisture(&sensor_id).await?;
                        Ok(values.len() as u64)
                    },
                )
                .await?,
            );
        }

        if wanted("sensor.view_history") {
            samples.push(
                measure("sensor.view_history", scale, "unrestricted", || async {
                    let page = sensors
                        .view_history(&sensor_id, first_page, None, None)
                        .await?;
                    Ok(page.items.len() as u64)
                })
                .await?,
            );
        }

        if wanted("sensor.last_plausible_values") {
            samples.push(
                measure(
                    "sensor.last_plausible_values",
                    scale,
                    "unrestricted",
                    || async {
                        let values = sensors.last_plausible_values(&sensor_id).await?;
                        Ok(values.len() as u64)
                    },
                )
                .await?,
            );
        }

        if wanted("sensor.quality_issues") {
            samples.push(
                measure("sensor.quality_issues", scale, "unrestricted", || async {
                    let issues = sensors.quality_issues(&sensor_id, 50).await?;
                    Ok(issues.len() as u64)
                })
                .await?,
            );
        }

        if wanted("sensor.soil_moisture_series") {
            let to = chrono::Utc::now();
            let from = to - chrono::Duration::days(30);
            samples.push(
                measure(
                    "sensor.soil_moisture_series",
                    scale,
                    "unrestricted",
                    || async {
                        let series = sensors
                            .soil_moisture_series(&sensor_id, from, to, SoilMoistureBucket::Day)
                            .await?;
                        Ok(series.len() as u64)
                    },
                )
                .await?,
            );
        }

        // The write path as a counter-check: insert rate into a growing time
        // series behaves differently from any read. The twenty rows this adds
        // are negligible against the seeded volume.
        if wanted("sensor.record") {
            let payload = serde_json::json!({
                "battery": 3.6,
                "temperature": 14.2,
                "soil_moisture": [
                    { "depth_cm": 40, "moisture_percent": 22.5 },
                    { "depth_cm": 80, "moisture_percent": 27.0 }
                ]
            });
            samples.push(
                measure("sensor.record", scale, "unrestricted", || async {
                    sensors
                        .record(SensorReadingDraft {
                            sensor_id: sensor_id.clone(),
                            data: payload.clone(),
                        })
                        .await?;
                    Ok(1)
                })
                .await?,
            );
        }
    }

    Ok(samples)
}
