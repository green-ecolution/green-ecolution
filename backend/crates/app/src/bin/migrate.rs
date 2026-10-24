use include_dir::{Dir, include_dir};
use server::configuration::get_configuration;
use sqlx::postgres::PgConnectOptions;
use sqlx::{Executor, PgPool, migrate::Migrator};
use std::env;

static MIGRATOR: Migrator = sqlx::migrate!("../../migrations");
static SEEDS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../seeds");

// Reset wipes app-owned objects in-place instead of DROP/CREATE DATABASE:
// managed Postgres (e.g. OVH) rejects connecting to template1 and forbids
// dropping databases. Extension objects (postgis' spatial_ref_sys, geometry
// type, pgcrypto) are skipped via pg_depend deptype='e' because the app user
// is not allowed to recreate them. Only the target `schema` is touched.
fn reset_sql(schema: &str) -> String {
    format!(
        r#"
DO $$
DECLARE
    r RECORD;
BEGIN
    FOR r IN
        SELECT c.relname
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = '{schema}'
          AND c.relkind = 'r'
          AND NOT EXISTS (
              SELECT 1 FROM pg_depend d WHERE d.objid = c.oid AND d.deptype = 'e'
          )
    LOOP
        EXECUTE format('DROP TABLE IF EXISTS %I.%I CASCADE', '{schema}', r.relname);
    END LOOP;

    FOR r IN
        SELECT t.typname
        FROM pg_type t
        JOIN pg_namespace n ON n.oid = t.typnamespace
        WHERE n.nspname = '{schema}'
          AND t.typtype = 'e'
          AND NOT EXISTS (
              SELECT 1 FROM pg_depend d WHERE d.objid = t.oid AND d.deptype = 'e'
          )
    LOOP
        EXECUTE format('DROP TYPE IF EXISTS %I.%I CASCADE', '{schema}', r.typname);
    END LOOP;
END $$;
"#
    )
}

// Reject anything that isn't a bare identifier: `schema` is interpolated into
// dynamic SQL (reset_sql / ensure_schema), so this is the injection guard.
fn validate_schema(schema: &str) -> Result<(), Box<dyn std::error::Error>> {
    let ok = !schema.is_empty()
        && schema
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        && !schema.as_bytes()[0].is_ascii_digit();
    if ok {
        Ok(())
    } else {
        Err(format!("invalid schema name: {schema:?} (expected [A-Za-z_][A-Za-z0-9_]*)").into())
    }
}

async fn ensure_schema(pool: &PgPool, schema: &str) -> Result<(), Box<dyn std::error::Error>> {
    pool.execute(format!(r#"CREATE SCHEMA IF NOT EXISTS "{schema}""#).as_str())
        .await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let cmd = args.next().unwrap_or_else(|| "up".into());
    let flags: Vec<String> = args.collect();
    let with_seed = flags.iter().any(|f| f == "--with-seed");
    let schema_override = flags
        .iter()
        .position(|f| f == "--schema")
        .and_then(|i| flags.get(i + 1))
        .cloned();

    let (app_options, schema) = resolve_options(schema_override)?;

    match cmd.as_str() {
        "up" | "run" => {
            let pool = PgPool::connect_with(app_options).await?;
            ensure_schema(&pool, &schema).await?;
            MIGRATOR.run(&pool).await?;
            println!("Migrations applied");
            if with_seed {
                run_seeds(&pool).await?;
            }
        }
        "info" | "status" => {
            let pool = PgPool::connect_with(app_options).await?;
            let applied: Vec<(i64,)> =
                sqlx::query_as("SELECT version FROM _sqlx_migrations ORDER BY version")
                    .fetch_all(&pool)
                    .await
                    .unwrap_or_default();
            let applied: std::collections::HashSet<i64> =
                applied.into_iter().map(|(v,)| v).collect();
            for m in MIGRATOR.iter() {
                let mark = if applied.contains(&m.version) {
                    "[x]"
                } else {
                    "[ ]"
                };
                println!("{mark} {} {}", m.version, m.description);
            }
        }
        "reset" => {
            let pool = PgPool::connect_with(app_options).await?;
            ensure_schema(&pool, &schema).await?;
            pool.execute(reset_sql(&schema).as_str()).await?;
            MIGRATOR.run(&pool).await?;
            println!("Schema '{schema}' wiped and migrated");
            if with_seed {
                run_seeds(&pool).await?;
            }
        }
        "seed" => {
            let pool = PgPool::connect_with(app_options).await?;
            run_seeds(&pool).await?;
        }
        "seed-down" => {
            let pool = PgPool::connect_with(app_options).await?;
            run_seed_downs(&pool).await?;
        }
        "-h" | "--help" | "help" => {
            print_usage();
        }
        other => {
            eprintln!("error: unknown command '{other}'\n");
            print_usage();
            std::process::exit(2);
        }
    }
    Ok(())
}

async fn run_seeds(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let mut files: Vec<_> = SEEDS
        .files()
        .filter(|f| {
            let name = f.path().file_name().and_then(|n| n.to_str()).unwrap_or("");
            name.ends_with(".sql") && !name.ends_with(".down.sql")
        })
        .collect();
    files.sort_by_key(|f| f.path().to_path_buf());

    if files.is_empty() {
        println!("No seed files found");
        return Ok(());
    }

    let count = files.len();
    for file in files {
        let name = file.path().display();
        let sql = file.contents_utf8().ok_or("seed file is not valid utf-8")?;
        println!("Applying seed: {name}");
        pool.execute(sql).await?;
    }
    println!("{count} seed file(s) applied");
    Ok(())
}

async fn run_seed_downs(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let mut files: Vec<_> = SEEDS
        .files()
        .filter(|f| {
            f.path()
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(".down.sql"))
        })
        .collect();
    // Reverse alphabetical order so dependent rows are removed before their parents.
    files.sort_by_key(|f| f.path().to_path_buf());
    files.reverse();

    if files.is_empty() {
        println!("No seed-down files found");
        return Ok(());
    }

    let count = files.len();
    for file in files {
        let name = file.path().display();
        let sql = file.contents_utf8().ok_or("seed file is not valid utf-8")?;
        println!("Reverting seed: {name}");
        pool.execute(sql).await?;
    }
    println!("{count} seed-down file(s) applied");
    Ok(())
}

fn resolve_options(
    schema_override: Option<String>,
) -> Result<(PgConnectOptions, String), Box<dyn std::error::Error>> {
    if let Ok(url) = env::var("DATABASE_URL") {
        let schema = schema_override
            .or_else(|| env::var("APP_DATABASE__SCHEMA").ok())
            .unwrap_or_else(|| "public".into());
        validate_schema(&schema)?;
        let options = url.parse::<PgConnectOptions>()?.options([(
            "search_path",
            server::configuration::search_path_for(&schema),
        )]);
        Ok((options, schema))
    } else {
        let mut db = get_configuration()?.database;
        if let Some(schema) = schema_override {
            db.schema = schema;
        }
        validate_schema(&db.schema)?;
        Ok((db.connection_options(), db.schema))
    }
}

fn print_usage() {
    eprintln!(
        "Usage: migrate [up|info|reset|seed|seed-down|help] [--with-seed] [--schema NAME]\n\
         \n\
         Commands:\n\
           up         Apply all pending migrations (default)\n\
           info       Show applied/pending migrations\n\
           reset      Wipe all app tables/types in the target schema, then migrate (DESTRUCTIVE)\n\
           seed       Apply all *.sql seed files (dev/demo data)\n\
           seed-down  Apply all *.down.sql files in reverse order to remove seed data\n\
           help       Show this message\n\
         \n\
         Flags:\n\
           --with-seed   After up/reset, also apply all seed files\n\
           --schema NAME Target schema (default: public, or APP_DATABASE__SCHEMA)\n\
         \n\
         Configuration: reads APP_DATABASE__* env vars (same as the server),\n\
         or DATABASE_URL as override."
    );
}
