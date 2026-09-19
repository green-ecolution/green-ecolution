//! The measured read paths.
//!
//! Every path goes through the real repository implementation rather than
//! hand-written SQL, so a measurement covers the same code the HTTP layer
//! calls, including any N+1 pattern a service happens to have.

use std::time::Instant;

use domain::{
    Id,
    authorization::Visibility,
    cluster::{TreeClusterSearchQuery, repository::TreeClusterReader},
    sensor::{SensorId, repository::SensorReadingReader},
    shared::{
        coordinates::Coordinate, distance::Distance, pagination::Pagination,
        watering_status::WateringStatus,
    },
    tree::{
        TreeSearchQuery,
        repository::TreeReader,
        sort::{TreeSort, TreeSortField},
    },
};
use server::{
    bench::scale::Scale,
    infra::{
        pg_cluster::PgTreeClusterRepository, pg_sensor::PgSensorRepository,
        pg_tree::PgTreeRepository,
    },
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
    "cluster.view_search",
    "cluster.view_markers",
    "cluster.boundaries",
    "cluster.statistics",
    "sensor.latest",
    "sensor.history",
    "sensor.latest_volumetric_moisture",
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

pub async fn measure_all(
    pool: &PgPool,
    scale: Scale,
    only: Option<&[String]>,
) -> Result<Vec<Sample>, Failure> {
    let wanted = |name: &str| only.is_none_or(|list| list.iter().any(|n| n == name));

    let trees = PgTreeRepository::new(pool.clone());
    let clusters = PgTreeClusterRepository::new(pool.clone());
    let sensors = PgSensorRepository::new(pool.clone(), chrono::Duration::days(1), 3);

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
                    Ok(page.items.len() as u64)
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
    }

    // Sensor readings carry no visibility filter, so measuring them per
    // variant would record the same number twice.
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
    }

    Ok(samples)
}
