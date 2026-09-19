//! Turns the raw samples into the one answer the campaign exists for: which
//! paths get slower faster than the data set grows.

use std::{collections::BTreeMap, fs::File, io::Write, path::Path};

use crate::measure::Sample;

/// A measurement on a shared machine carries noise, so a path only counts as
/// superlinear once it is clearly past the row growth rather than a few per
/// cent over it.
const SUPERLINEAR_TOLERANCE: f64 = 1.3;

/// Only the unrestricted variant forms the curve. The scoped one answers a
/// different question (how much does the filter save) and mixing the two would
/// compare a full table against a subtree.
const CURVE_VISIBILITY: &str = "unrestricted";

#[derive(Debug, Clone)]
pub struct Growth {
    pub path: String,
    pub from_scale: String,
    pub to_scale: String,
    pub row_factor: f64,
    pub time_factor: f64,
    pub superlinear: bool,
}

pub fn growth(samples: &[Sample], row_counts: &[(String, u64)]) -> Vec<Growth> {
    let mut out = Vec::new();
    let mut paths: Vec<&str> = samples.iter().map(|s| s.path.as_str()).collect();
    paths.sort_unstable();
    paths.dedup();

    for path in paths {
        for window in row_counts.windows(2) {
            let (from_scale, from_rows) = &window[0];
            let (to_scale, to_rows) = &window[1];

            let (Some(from), Some(to)) = (
                find(samples, path, from_scale),
                find(samples, path, to_scale),
            ) else {
                continue;
            };

            let row_factor = *to_rows as f64 / *from_rows as f64;
            let time_factor = to.median_ms / from.median_ms;

            out.push(Growth {
                path: path.to_string(),
                from_scale: from_scale.clone(),
                to_scale: to_scale.clone(),
                row_factor,
                time_factor,
                superlinear: time_factor > row_factor * SUPERLINEAR_TOLERANCE,
            });
        }
    }
    out
}

fn find<'a>(samples: &'a [Sample], path: &str, scale: &str) -> Option<&'a Sample> {
    samples
        .iter()
        .find(|s| s.path == path && s.scale == scale && s.visibility == CURVE_VISIBILITY)
}

pub fn write_markdown(
    growths: &[Growth],
    samples: &[Sample],
    seq_scans: &[(String, String)],
    out: &Path,
) -> std::io::Result<()> {
    let mut file = File::create(out.join("report.md"))?;

    writeln!(file, "# Skalierungsbericht\n")?;
    writeln!(
        file,
        "Die Absolutwerte stammen aus einer lokalen Postgres-Instanz und sind \
nicht auf Produktionshardware uebertragbar. Uebertragbar ist die Form der \
Kurve: ob ein Pfad mit der Datenmenge mitwaechst oder schneller.\n"
    )?;
    writeln!(
        file,
        "Die Kurve wird aus den Messungen ohne Scope-Filter gebildet. Die \
Scope-Variante steht in der Gesamttabelle und beantwortet eine andere Frage, \
naemlich wie viel der Filter einspart.\n"
    )?;

    writeln!(file, "## Ueberlinear wachsende Pfade\n")?;
    let flagged: Vec<&Growth> = growths.iter().filter(|g| g.superlinear).collect();
    if flagged.is_empty() {
        writeln!(
            file,
            "Keiner. Jeder gemessene Pfad bleibt innerhalb des Zeilenwachstums.\n"
        )?;
    } else {
        writeln!(file, "| Pfad | Stufe | Zeilen-Faktor | Zeit-Faktor |")?;
        writeln!(file, "| --- | --- | --- | --- |")?;
        for g in flagged {
            writeln!(
                file,
                "| `{}` | {} -> {} | {:.1}x | {:.1}x |",
                g.path, g.from_scale, g.to_scale, g.row_factor, g.time_factor
            )?;
        }
        writeln!(file)?;
    }

    writeln!(file, "## Sequentielle Scans auf der groessten Stufe\n")?;
    if seq_scans.is_empty() {
        writeln!(
            file,
            "Keine. Jeder erfasste Plan nutzt einen Index auf der jeweils \
grossen Tabelle.\n"
        )?;
    } else {
        writeln!(file, "| Plan | Tabelle |")?;
        writeln!(file, "| --- | --- |")?;
        for (plan, table) in seq_scans {
            writeln!(file, "| `{plan}` | `{table}` |")?;
        }
        writeln!(file)?;
    }

    writeln!(file, "## Wachstum je Pfad\n")?;
    writeln!(
        file,
        "| Pfad | Stufe | Zeilen-Faktor | Zeit-Faktor | Bewertung |"
    )?;
    writeln!(file, "| --- | --- | --- | --- | --- |")?;
    for g in growths {
        let verdict = if g.superlinear {
            "ueberlinear"
        } else if g.time_factor < g.row_factor * 0.5 {
            "deutlich sublinear"
        } else {
            "im Rahmen"
        };
        writeln!(
            file,
            "| `{}` | {} -> {} | {:.1}x | {:.1}x | {} |",
            g.path, g.from_scale, g.to_scale, g.row_factor, g.time_factor, verdict
        )?;
    }
    writeln!(file)?;

    writeln!(file, "## Alle Messungen\n")?;
    writeln!(
        file,
        "| Pfad | Stufe | Sichtbarkeit | Median (ms) | p95 (ms) | Zeilen |"
    )?;
    writeln!(file, "| --- | --- | --- | --- | --- | --- |")?;
    let mut by_path: BTreeMap<&str, Vec<&Sample>> = BTreeMap::new();
    for s in samples {
        by_path.entry(s.path.as_str()).or_default().push(s);
    }
    for (_, group) in by_path {
        for s in group {
            writeln!(
                file,
                "| `{}` | {} | {} | {:.2} | {:.2} | {} |",
                s.path, s.scale, s.visibility, s.median_ms, s.p95_ms, s.rows
            )?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::measure::summarize;

    fn sample(path: &str, scale: &str, median_ms: f64) -> Sample {
        summarize(path, scale, "unrestricted", vec![median_ms], 25)
    }

    fn two_scales() -> Vec<(String, u64)> {
        vec![("xs".to_string(), 1_000), ("s".to_string(), 10_000)]
    }

    #[test]
    fn linear_growth_is_not_flagged() {
        // ten times the rows, ten times the time
        let samples = vec![sample("p", "xs", 1.0), sample("p", "s", 10.0)];

        let growths = growth(&samples, &two_scales());

        assert_eq!(growths.len(), 1);
        assert!(!growths[0].superlinear, "{:?}", growths[0]);
    }

    #[test]
    fn quadratic_growth_is_flagged() {
        // ten times the rows, a hundred times the time
        let samples = vec![sample("p", "xs", 1.0), sample("p", "s", 100.0)];

        let growths = growth(&samples, &two_scales());

        assert!(growths[0].superlinear, "{:?}", growths[0]);
        assert!((growths[0].time_factor - 100.0).abs() < 0.001);
    }

    #[test]
    fn sublinear_growth_is_not_flagged() {
        // ten times the rows, twice the time: an index doing its job
        let samples = vec![sample("p", "xs", 1.0), sample("p", "s", 2.0)];

        assert!(!growth(&samples, &two_scales())[0].superlinear);
    }

    #[test]
    fn a_single_scale_yields_no_growth_rows() {
        let samples = vec![sample("p", "xs", 1.0)];
        let rows = vec![("xs".to_string(), 1_000)];

        assert!(growth(&samples, &rows).is_empty());
    }

    #[test]
    fn a_path_measured_at_only_one_scale_is_skipped() {
        let samples = vec![
            sample("everywhere", "xs", 1.0),
            sample("everywhere", "s", 10.0),
            sample("once", "xs", 1.0),
        ];

        let growths = growth(&samples, &two_scales());

        assert_eq!(growths.len(), 1);
        assert_eq!(growths[0].path, "everywhere");
    }

    #[test]
    fn scoped_samples_do_not_mix_into_the_unrestricted_curve() {
        let mut samples = vec![sample("p", "xs", 1.0), sample("p", "s", 10.0)];
        samples.push(summarize("p", "s", "scoped", vec![500.0], 25));

        let growths = growth(&samples, &two_scales());

        assert_eq!(growths.len(), 1);
        assert!((growths[0].time_factor - 10.0).abs() < 0.001);
    }
}
