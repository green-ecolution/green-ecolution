use anyhow::{Context, Result};
use chrono::{Datelike, Utc};
use serde_json::{Value, json};
#[allow(unused_imports)]
use server::configuration::get_configuration;
use sqlx::{PgPool, Row};
use std::env;
#[allow(unused_imports)]
use uuid::Uuid;

const DEFAULT_PROVIDER: &str = "tbz-baumkataster";

const SOURCE_QUERY: &str = r#"
    SELECT
        "OBJECTID"::int                AS objectid,
        "BAUMNUMMER"::text             AS baumnummer,
        "GATTUNG"::text                AS gattung,
        "PFLANZJAHR"::int              AS pflanzjahr,
        "RECHTSWERT"::double precision AS rechtswert,
        "HOCHWERT"::double precision   AS hochwert
    FROM metadata_baum.baumkataster
    WHERE "PFLANZJAHR"::int >= $1
"#;

#[allow(dead_code)] // dry_run used in the upsert task
struct Args {
    dry_run: bool,
    year_cutoff: Option<i32>,
    provider: String,
}

/// Raw row read from the cadastre. All casts happen in `SOURCE_QUERY` so the
/// external schema's exact column types (int vs text) don't matter here.
struct KatasterRow {
    objectid: i32,
    baumnummer: Option<String>,
    gattung: Option<String>,
    pflanzjahr: i32,
    rechtswert: Option<f64>,
    hochwert: Option<f64>,
}

/// Target-shaped fields ready to upsert. Holds raw GK3 coordinates; the SQL
/// reprojects them via ST_Transform.
#[allow(dead_code)] // fields consumed in the upsert task
struct TreeImport {
    number: String,
    species: String,
    planting_year: i32,
    rechtswert: f64,
    hochwert: f64,
    additional_info: Value,
}

/// Pure mapping cadastre row → import. Returns `None` when the point is
/// unusable (NULL or zero coordinates) so the caller can skip it.
fn map_row(row: &KatasterRow, provider: &str) -> Option<TreeImport> {
    let (rechtswert, hochwert) = match (row.rechtswert, row.hochwert) {
        (Some(r), Some(h)) if r != 0.0 && h != 0.0 => (r, h),
        _ => return None,
    };

    let number = row
        .baumnummer
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| row.objectid.to_string());

    let species = row
        .gattung
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| "Unbekannt".to_string());

    let additional_info = json!({
        "kataster_objectid": row.objectid,
        "source": provider,
        "gattung": row.gattung,
        "baumnummer": number,
    });

    Some(TreeImport {
        number,
        species,
        planting_year: row.pflanzjahr,
        rechtswert,
        hochwert,
        additional_info,
    })
}

fn read_row(row: &sqlx::postgres::PgRow) -> Result<KatasterRow> {
    Ok(KatasterRow {
        objectid: row.try_get("objectid").context("column OBJECTID")?,
        baumnummer: row.try_get("baumnummer").context("column BAUMNUMMER")?,
        gattung: row.try_get("gattung").context("column GATTUNG")?,
        pflanzjahr: row.try_get("pflanzjahr").context("column PFLANZJAHR")?,
        rechtswert: row.try_get("rechtswert").context("column RECHTSWERT")?,
        hochwert: row.try_get("hochwert").context("column HOCHWERT")?,
    })
}

fn parse_args() -> Result<Args> {
    let mut dry_run = false;
    let mut year_cutoff = None;
    let mut provider = DEFAULT_PROVIDER.to_string();
    let mut it = env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--dry-run" => dry_run = true,
            "--year-cutoff" => {
                let v = it.next().context("--year-cutoff needs a value")?;
                year_cutoff = Some(v.parse().context("--year-cutoff must be an integer")?);
            }
            "--provider" => provider = it.next().context("--provider needs a value")?,
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            other => anyhow::bail!("unknown argument: {other}"),
        }
    }
    Ok(Args {
        dry_run,
        year_cutoff,
        provider,
    })
}

fn print_usage() {
    eprintln!(
        "import-kataster — Flensburg Baumkataster -> Green Ecolution\n\n\
         Env:\n  KATASTER_SOURCE_URL   source Postgres connection string (required)\n\
         Target DB resolved from Green Ecolution config (config/*.yaml + APP_DATABASE__* env)\n\n\
         Flags:\n  --dry-run             run everything in a transaction, then roll back\n  \
           --year-cutoff <N>     only PFLANZJAHR >= N (default: current year - 3)\n  \
           --provider <id>       provider value + match key (default: tbz-baumkataster)\n"
    );
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = parse_args()?;
    let cutoff = args.year_cutoff.unwrap_or_else(|| Utc::now().year() - 3);

    let source_url = env::var("KATASTER_SOURCE_URL")
        .context("set KATASTER_SOURCE_URL to the source Postgres connection string")?;
    let source = PgPool::connect(&source_url)
        .await
        .context("connecting to source DB")?;

    let rows = sqlx::query(SOURCE_QUERY)
        .bind(cutoff)
        .fetch_all(&source)
        .await
        .context("querying metadata_baum.baumkataster")?;

    let (mut scanned, mut mappable, mut skipped) = (0usize, 0usize, 0usize);
    for row in &rows {
        scanned += 1;
        let kr = read_row(row)?;
        match map_row(&kr, &args.provider) {
            Some(_) => mappable += 1,
            None => {
                skipped += 1;
                eprintln!("skip objectid={}: missing/zero coordinates", kr.objectid);
            }
        }
    }

    println!("[read-only] cutoff>={cutoff} · geprüft {scanned} · abbildbar {mappable} · übersprungen {skipped}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> KatasterRow {
        KatasterRow {
            objectid: 30492,
            baumnummer: Some(" 10".into()),
            gattung: Some("Sorbus".into()),
            pflanzjahr: 2024,
            rechtswert: Some(3527374.24),
            hochwert: Some(6074935.94),
        }
    }

    #[test]
    fn trims_baumnummer() {
        let imp = map_row(&sample(), DEFAULT_PROVIDER).unwrap();
        assert_eq!(imp.number, "10");
    }

    #[test]
    fn blank_baumnummer_falls_back_to_objectid() {
        let mut r = sample();
        r.baumnummer = Some("   ".into());
        assert_eq!(map_row(&r, DEFAULT_PROVIDER).unwrap().number, "30492");
    }

    #[test]
    fn null_baumnummer_falls_back_to_objectid() {
        let mut r = sample();
        r.baumnummer = None;
        assert_eq!(map_row(&r, DEFAULT_PROVIDER).unwrap().number, "30492");
    }

    #[test]
    fn null_gattung_becomes_unbekannt() {
        let mut r = sample();
        r.gattung = None;
        assert_eq!(map_row(&r, DEFAULT_PROVIDER).unwrap().species, "Unbekannt");
    }

    #[test]
    fn missing_coordinate_is_skipped() {
        let mut r = sample();
        r.rechtswert = None;
        assert!(map_row(&r, DEFAULT_PROVIDER).is_none());
    }

    #[test]
    fn zero_coordinate_is_skipped() {
        let mut r = sample();
        r.hochwert = Some(0.0);
        assert!(map_row(&r, DEFAULT_PROVIDER).is_none());
    }

    #[test]
    fn additional_info_carries_objectid_and_provider() {
        let imp = map_row(&sample(), "custom-provider").unwrap();
        assert_eq!(imp.additional_info["kataster_objectid"], 30492);
        assert_eq!(imp.additional_info["source"], "custom-provider");
    }
}
