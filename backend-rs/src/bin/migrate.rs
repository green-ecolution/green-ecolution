use green_ecolution::configuration::get_configuration;
use include_dir::{Dir, include_dir};
use secrecy::ExposeSecret;
use sqlx::postgres::PgConnectOptions;
use sqlx::{Connection, Executor, PgConnection, PgPool, migrate::Migrator};
use std::env;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");
static SEEDS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/seeds");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let cmd = args.next().unwrap_or_else(|| "up".into());
    let flags: Vec<String> = args.collect();
    let with_seed = flags.iter().any(|f| f == "--with-seed");

    let (admin_options, app_options, db_name) = resolve_options()?;

    match cmd.as_str() {
        "up" | "run" => {
            let pool = PgPool::connect_with(app_options).await?;
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
            let mut admin = PgConnection::connect_with(&admin_options).await?;
            let terminate = format!(
                "SELECT pg_terminate_backend(pid) FROM pg_stat_activity \
                 WHERE datname = '{db_name}' AND pid <> pg_backend_pid()"
            );
            let _ = admin.execute(terminate.as_str()).await;
            admin
                .execute(format!(r#"DROP DATABASE IF EXISTS "{db_name}""#).as_str())
                .await?;
            admin
                .execute(format!(r#"CREATE DATABASE "{db_name}""#).as_str())
                .await?;
            admin.close().await?;

            let pool = PgPool::connect_with(app_options).await?;
            MIGRATOR.run(&pool).await?;
            println!("Database '{db_name}' dropped, recreated, and migrated");
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

fn resolve_options() -> Result<(PgConnectOptions, PgConnectOptions, String), Box<dyn std::error::Error>>
{
    if let Ok(url) = env::var("DATABASE_URL") {
        let app: PgConnectOptions = url.parse()?;
        let db_name = app
            .get_database()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "postgres".into());
        let admin = app.clone().database(admin_db(&db_name));
        Ok((admin, app, db_name))
    } else {
        let cfg = get_configuration()?;
        let app = PgConnectOptions::new()
            .host(&cfg.database.host)
            .port(cfg.database.port)
            .username(&cfg.database.username)
            .password(cfg.database.password.expose_secret())
            .database(&cfg.database.database_name)
            .ssl_mode(if cfg.database.require_ssl {
                sqlx::postgres::PgSslMode::Require
            } else {
                sqlx::postgres::PgSslMode::Prefer
            });
        let admin = app.clone().database(admin_db(&cfg.database.database_name));
        Ok((admin, app, cfg.database.database_name))
    }
}

// Pick a system DB to connect to for DROP/CREATE that is guaranteed not to
// be the target DB itself (Postgres refuses to drop a DB you're connected to).
fn admin_db(target: &str) -> &'static str {
    if target == "template1" {
        "postgres"
    } else {
        "template1"
    }
}

fn print_usage() {
    eprintln!(
        "Usage: migrate [up|info|reset|seed|seed-down|help] [--with-seed]\n\
         \n\
         Commands:\n\
           up         Apply all pending migrations (default)\n\
           info       Show applied/pending migrations\n\
           reset      Drop and recreate the database, then migrate (DESTRUCTIVE)\n\
           seed       Apply all *.sql seed files (dev/demo data)\n\
           seed-down  Apply all *.down.sql files in reverse order to remove seed data\n\
           help       Show this message\n\
         \n\
         Flags:\n\
           --with-seed   After up/reset, also apply all seed files\n\
         \n\
         Configuration: reads APP_DATABASE__* env vars (same as the server),\n\
         or DATABASE_URL as override."
    );
}
