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
            capture_plans(&pool, scale.name(), &out).await?;

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

                samples.extend(paths::measure_all(&pool, current, None).await?);

                capture_plans(&pool, current.name(), &out).await?;

                println!("scale {} done: {counts:?}", current.name());
            }

            environment::capture(&pool, &out).await?;
            measure::write_csv(&samples, &out.join("results.csv"))?;

            println!(
                "measured {} paths across {} scales into {}",
                samples.len(),
                Scale::ALL.len(),
                out.display()
            );
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

/// Captures the plan probes for one scale. Both halves of the tree list are
/// interesting: the count decides whether the whole table is read, the row
/// query whether the sort can use an index.
async fn capture_plans(
    pool: &PgPool,
    scale: &str,
    out: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    explain::capture(
        pool,
        &format!("tree.view_search.count_{scale}"),
        paths::TREE_COUNT_SQL,
        out,
    )
    .await?;
    explain::capture(
        pool,
        &format!("tree.view_search.rows_{scale}"),
        paths::TREE_ROWS_SQL,
        out,
    )
    .await
}
