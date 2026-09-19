//! Large-data benchmark harness. Seeds a dedicated Postgres instance in steps
//! and measures the real repository implementations against it.
//!
//! Never point this at a development database: `--database-url` is required
//! and has no default for exactly that reason.

mod cli;
mod environment;
mod explain;
mod measure;
mod paths;
mod report;

use server::bench::{scale::Scale, seed};
use sqlx::{PgPool, postgres::PgPoolOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = match cli::parse(std::env::args().skip(1)) {
        Ok(command) => command,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    };

    match command {
        cli::Command::Seed {
            database_url,
            scale,
            history_days,
            seed,
            append,
        } => {
            let pool = connect(&database_url).await?;
            if !append {
                ensure_empty(&pool).await?;
            }
            println!(
                "seeding scale {} ({} trees, {history_days} days of history, seed {seed})",
                scale.name(),
                scale.trees()
            );
            let counts = seed::seed_core(
                &pool,
                &seed::SeedPlan {
                    scale,
                    history_days,
                    seed,
                },
            )
            .await?;
            seed::analyze(&pool).await?;
            println!("{counts:?}");
            Ok(())
        }
        cli::Command::Run {
            database_url,
            out,
            only,
        } => {
            let pool = connect(&database_url).await?;
            if let Some(list) = &only {
                reject_unknown_paths(list)?;
            }
            std::fs::create_dir_all(&out)?;

            let scale = detect_scale(&pool).await?;
            let samples = paths::measure_all(&pool, scale, only.as_deref()).await?;
            measure::write_csv(&samples, &out.join("results.csv"))?;
            environment::capture(&pool, &out).await?;
            explain::capture(
                &pool,
                &format!("tree.view_search.count_{}", scale.name()),
                paths::TREE_VIEW_SEARCH_SQL,
                &out,
            )
            .await?;

            println!(
                "measured {} paths at scale {} into {}",
                samples.len(),
                scale.name(),
                out.display()
            );
            Ok(())
        }
        cli::Command::Campaign {
            database_url,
            out,
            history_days,
            seed: seed_value,
        } => {
            let pool = connect(&database_url).await?;
            ensure_empty(&pool).await?;
            std::fs::create_dir_all(&out)?;

            let mut samples = Vec::new();
            let mut row_counts = Vec::new();

            for current in Scale::ALL {
                // Grow into the next step rather than rebuilding: the seeder
                // tops every table up to the new size.
                let counts = seed::seed_core(
                    &pool,
                    &seed::SeedPlan {
                        scale: current,
                        history_days,
                        seed: seed_value,
                    },
                )
                .await?;
                seed::analyze(&pool).await?;

                row_counts.push((current.name().to_string(), counts.trees as u64));
                samples.extend(paths::measure_all(&pool, current, None).await?);

                let plan_name = format!("tree.view_search.count_{}", current.name());
                explain::capture(&pool, &plan_name, paths::TREE_VIEW_SEARCH_SQL, &out).await?;

                println!("scale {} done: {counts:?}", current.name());
            }

            environment::capture(&pool, &out).await?;
            measure::write_csv(&samples, &out.join("results.csv"))?;

            let largest = Scale::ALL.last().expect("Scale::ALL is never empty");
            let seq_scans = seq_scans_at(&out, largest.name())?;
            let growths = report::growth(&samples, &row_counts);
            report::write_markdown(&growths, &samples, &seq_scans, &out)?;

            println!("report written to {}", out.join("report.md").display());
            Ok(())
        }
    }
}

/// One connection throughout: the measurements report latency, and a pool
/// would report connection scheduling on top of it.
async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(1)
        .connect(database_url)
        .await
}

/// A non-empty target is almost always somebody's dev database. Refusing here
/// is cheaper than explaining where their data went.
async fn ensure_empty(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let trees: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM trees")
        .fetch_one(pool)
        .await?;
    if trees > 0 {
        return Err(format!(
            "target database already holds {trees} trees; pass --append to add to it"
        )
        .into());
    }
    Ok(())
}

/// `run` can be pointed at a database somebody else seeded, so the scale is
/// read back from the row count instead of taken on trust.
async fn detect_scale(pool: &PgPool) -> Result<Scale, Box<dyn std::error::Error>> {
    let trees: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM trees")
        .fetch_one(pool)
        .await?;
    Ok(Scale::ALL
        .into_iter()
        .min_by_key(|s| (s.trees() - trees).abs())
        .unwrap_or(Scale::Xs))
}

/// A typo in `--only` would otherwise silently measure nothing at all.
fn reject_unknown_paths(requested: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    for name in requested {
        if !paths::NAMES.contains(&name.as_str()) {
            return Err(format!(
                "unknown path '{name}'; known paths: {}",
                paths::NAMES.join(", ")
            )
            .into());
        }
    }
    Ok(())
}

/// Reads back the plans captured at the largest scale and reports which of the
/// big tables are still being scanned sequentially there. A plan that still
/// scans at the top of the range says more than any single timing does.
fn seq_scans_at(
    out: &std::path::Path,
    scale: &str,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    const BIG_TABLES: [&str; 3] = ["trees", "tree_clusters", "sensor_data"];

    let plans_dir = out.join("plans");
    if !plans_dir.exists() {
        return Ok(Vec::new());
    }

    let mut found = Vec::new();
    for entry in std::fs::read_dir(&plans_dir)? {
        let path = entry?.path();
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        if !name.ends_with(&format!("_{scale}")) {
            continue;
        }
        let plan: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)?;
        for table in BIG_TABLES {
            if explain::has_seq_scan_on(&plan, table) {
                found.push((name.clone(), table.to_string()));
            }
        }
    }
    found.sort();
    Ok(found)
}
