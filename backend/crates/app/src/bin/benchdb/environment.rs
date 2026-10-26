//! What the numbers were measured on.
//!
//! Absolute timings from a laptop's Docker Postgres do not transfer to
//! production hardware. Recording the context is what keeps a stored run
//! interpretable months later, and what stops two runs from being compared
//! when they should not be.

use std::path::Path;

use sqlx::PgPool;

type Failure = Box<dyn std::error::Error>;

pub async fn capture(pool: &PgPool, out: &Path) -> Result<(), Failure> {
    let version: String = sqlx::query_scalar("SELECT version()")
        .fetch_one(pool)
        .await?;
    let shared_buffers: String = sqlx::query_scalar("SHOW shared_buffers")
        .fetch_one(pool)
        .await?;
    let work_mem: String = sqlx::query_scalar("SHOW work_mem").fetch_one(pool).await?;
    let effective_cache_size: String = sqlx::query_scalar("SHOW effective_cache_size")
        .fetch_one(pool)
        .await?;

    let table_sizes: Vec<(String, i64)> = sqlx::query_as(
        r#"SELECT c.relname::text, pg_total_relation_size(c.oid)
           FROM pg_class c
           JOIN pg_namespace n ON n.oid = c.relnamespace
           WHERE n.nspname = 'public' AND c.relkind = 'r'
           ORDER BY pg_total_relation_size(c.oid) DESC
           LIMIT 10"#,
    )
    .fetch_all(pool)
    .await?;

    let payload = serde_json::json!({
        "postgres_version": version,
        "shared_buffers": shared_buffers,
        "work_mem": work_mem,
        "effective_cache_size": effective_cache_size,
        "cpus": std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0),
        "captured_at": chrono::Utc::now().to_rfc3339(),
        "largest_tables_bytes": table_sizes
            .into_iter()
            .map(|(name, bytes)| serde_json::json!({ "table": name, "bytes": bytes }))
            .collect::<Vec<_>>(),
    });

    std::fs::write(
        out.join("environment.json"),
        serde_json::to_vec_pretty(&payload)?,
    )?;
    Ok(())
}
