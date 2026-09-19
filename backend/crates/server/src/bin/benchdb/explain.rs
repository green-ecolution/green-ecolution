//! Query-plan capture.
//!
//! A plan often says more than a timing does: a sequential scan still present
//! at the largest scale is a finding regardless of how many milliseconds it
//! happened to take on this machine.

use std::path::Path;

use sqlx::PgPool;

type Failure = Box<dyn std::error::Error>;

pub async fn capture(pool: &PgPool, name: &str, sql: &str, out_dir: &Path) -> Result<(), Failure> {
    let plans_dir = out_dir.join("plans");
    std::fs::create_dir_all(&plans_dir)?;

    // The probes carry their filters as literals, so there is nothing to bind
    // and the planner sees the same constants a real request produces.
    let statement = format!("EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON) {sql}");
    let plan: serde_json::Value = sqlx::query_scalar(&statement).fetch_one(pool).await?;

    std::fs::write(
        plans_dir.join(format!("{name}.json")),
        serde_json::to_vec_pretty(&plan)?,
    )?;
    Ok(())
}
