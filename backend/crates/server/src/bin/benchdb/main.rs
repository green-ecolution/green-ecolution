//! Large-data benchmark harness. Seeds a dedicated Postgres instance in steps
//! and measures the real repository implementations against it.
//!
//! Never point this at a development database: `--database-url` is required
//! and has no default for exactly that reason.

mod cli;

use server::bench::scale::Scale;
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
            Ok(())
        }
        cli::Command::Run { .. } | cli::Command::Campaign { .. } => {
            eprintln!("not implemented yet");
            std::process::exit(1);
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
#[allow(dead_code)] // reason: used by the `run` subcommand, landing in a follow-up commit
async fn detect_scale(pool: &PgPool) -> Result<Scale, Box<dyn std::error::Error>> {
    let trees: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM trees")
        .fetch_one(pool)
        .await?;
    Ok(Scale::ALL
        .into_iter()
        .min_by_key(|s| (s.trees() - trees).abs())
        .unwrap_or(Scale::Xs))
}
