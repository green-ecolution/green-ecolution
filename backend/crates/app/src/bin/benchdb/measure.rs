use std::{fs::File, io::Write, path::Path};

/// Warm-up runs before a measurement starts. The first call against a cold
/// buffer cache measures the disk, not the query.
pub const WARMUP: usize = 5;

/// Counted runs per measurement. Enough for a median and a p95 that were both
/// actually observed, few enough that a whole campaign stays finite.
pub const ITERATIONS: usize = 20;

#[derive(Debug, Clone)]
pub struct Sample {
    pub path: String,
    pub scale: String,
    pub visibility: String,
    pub min_ms: f64,
    pub median_ms: f64,
    pub p95_ms: f64,
    pub max_ms: f64,
    pub rows: u64,
}

pub fn summarize(
    path: &str,
    scale: &str,
    visibility: &str,
    mut durations: Vec<f64>,
    rows: u64,
) -> Sample {
    durations.sort_by(|a, b| a.partial_cmp(b).expect("elapsed times are never NaN"));
    Sample {
        path: path.to_string(),
        scale: scale.to_string(),
        visibility: visibility.to_string(),
        min_ms: durations[0],
        median_ms: percentile(&durations, 0.5),
        p95_ms: percentile(&durations, 0.95),
        max_ms: durations[durations.len() - 1],
        rows,
    }
}

/// Nearest-rank on the sorted slice. No interpolation: with 20 samples an
/// interpolated p95 would invent a value that was never measured.
fn percentile(sorted: &[f64], q: f64) -> f64 {
    let rank = (q * sorted.len() as f64).ceil().max(1.0) as usize;
    sorted[rank.min(sorted.len()) - 1]
}

pub fn write_csv(samples: &[Sample], path: &Path) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(
        file,
        "path,scale,visibility,min_ms,median_ms,p95_ms,max_ms,rows"
    )?;
    for s in samples {
        writeln!(
            file,
            "{},{},{},{:.3},{:.3},{:.3},{:.3},{}",
            s.path, s.scale, s.visibility, s.min_ms, s.median_ms, s.p95_ms, s.max_ms, s.rows
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentiles_come_from_the_sorted_samples() {
        let durations: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        let sample = summarize("tree.view_search", "m", "unrestricted", durations, 25);

        assert_eq!(sample.min_ms, 1.0);
        assert_eq!(sample.max_ms, 100.0);
        assert_eq!(sample.median_ms, 50.0);
        assert_eq!(sample.p95_ms, 95.0);
        assert_eq!(sample.rows, 25);
    }

    #[test]
    fn unsorted_input_is_ordered_before_the_percentiles_are_read() {
        let sample = summarize("p", "xs", "unrestricted", vec![9.0, 1.0, 5.0], 1);

        assert_eq!(sample.min_ms, 1.0);
        assert_eq!(sample.median_ms, 5.0);
        assert_eq!(sample.max_ms, 9.0);
    }

    #[test]
    fn a_single_measurement_is_its_own_percentile() {
        let sample = summarize("tree.by_id", "xs", "unrestricted", vec![4.0], 1);

        assert_eq!(sample.min_ms, 4.0);
        assert_eq!(sample.median_ms, 4.0);
        assert_eq!(sample.p95_ms, 4.0);
        assert_eq!(sample.max_ms, 4.0);
    }

    #[test]
    fn csv_carries_a_header_and_one_line_per_sample() {
        let dir = std::env::temp_dir().join(format!("benchdb-csv-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir must be creatable");
        let path = dir.join("results.csv");

        let samples = vec![
            summarize("a", "xs", "unrestricted", vec![1.0], 3),
            summarize("b", "xs", "scoped", vec![2.0], 4),
        ];
        write_csv(&samples, &path).expect("writing must succeed");

        let written = std::fs::read_to_string(&path).expect("the file must exist");
        let lines: Vec<&str> = written.lines().collect();

        assert_eq!(lines.len(), 3);
        assert!(lines[0].starts_with("path,scale,visibility"));
        assert!(lines[1].starts_with("a,xs,unrestricted"));
        assert!(lines[2].starts_with("b,xs,scoped"));

        std::fs::remove_dir_all(&dir).ok();
    }
}
