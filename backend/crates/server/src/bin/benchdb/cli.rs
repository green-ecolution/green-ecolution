use std::path::PathBuf;

use server::bench::scale::Scale;

#[derive(Debug)]
// reason: the Run and Campaign payloads are consumed once the measuring and
// campaign subcommands land; parsing them already keeps the CLI contract in
// one place.
#[allow(dead_code)]
pub enum Command {
    Seed {
        database_url: String,
        scale: Scale,
        history_days: i64,
        seed: u64,
        append: bool,
    },
    Run {
        database_url: String,
        out: PathBuf,
        only: Option<Vec<String>>,
    },
    Campaign {
        database_url: String,
        out: PathBuf,
        history_days: i64,
        seed: u64,
    },
}

pub const USAGE: &str = "\
benchdb seed     --database-url <url> --scale <xs|s|m|l> [--history-days N] [--seed N] [--append]
benchdb run      --database-url <url> --out <dir> [--only <path,path>]
benchdb campaign --database-url <url> --out <dir> [--history-days N] [--seed N]";

pub const DEFAULT_HISTORY_DAYS: i64 = 730;
pub const DEFAULT_SEED: u64 = 42;

/// Hand-rolled rather than `clap`: the workspace does not carry an argument
/// parser, and six flags do not justify adding one.
pub fn parse(mut args: impl Iterator<Item = String>) -> Result<Command, String> {
    let subcommand = args
        .next()
        .ok_or_else(|| format!("missing subcommand\n\n{USAGE}"))?;

    let mut database_url: Option<String> = None;
    let mut scale: Option<Scale> = None;
    let mut out: Option<PathBuf> = None;
    let mut only: Option<Vec<String>> = None;
    let mut history_days = DEFAULT_HISTORY_DAYS;
    let mut seed = DEFAULT_SEED;
    let mut append = false;

    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--database-url" => database_url = Some(value(&mut args, "--database-url")?),
            "--scale" => scale = Some(Scale::parse(&value(&mut args, "--scale")?)?),
            "--out" => out = Some(PathBuf::from(value(&mut args, "--out")?)),
            "--only" => {
                only = Some(
                    value(&mut args, "--only")?
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect(),
                )
            }
            "--history-days" => history_days = parse_number(&value(&mut args, "--history-days")?)?,
            "--seed" => seed = parse_number(&value(&mut args, "--seed")?)?,
            "--append" => append = true,
            other => return Err(format!("unknown flag '{other}'\n\n{USAGE}")),
        }
    }

    let database_url =
        database_url.ok_or_else(|| format!("--database-url is required\n\n{USAGE}"))?;

    match subcommand.as_str() {
        "seed" => Ok(Command::Seed {
            database_url,
            scale: scale.ok_or_else(|| format!("--scale is required\n\n{USAGE}"))?,
            history_days,
            seed,
            append,
        }),
        "run" => Ok(Command::Run {
            database_url,
            out: out.ok_or_else(|| format!("--out is required\n\n{USAGE}"))?,
            only,
        }),
        "campaign" => Ok(Command::Campaign {
            database_url,
            out: out.ok_or_else(|| format!("--out is required\n\n{USAGE}"))?,
            history_days,
            seed,
        }),
        other => Err(format!("unknown subcommand '{other}'\n\n{USAGE}")),
    }
}

fn value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    args.next().ok_or_else(|| format!("{flag} expects a value"))
}

fn parse_number<T: std::str::FromStr>(raw: &str) -> Result<T, String> {
    raw.parse().map_err(|_| format!("'{raw}' is not a number"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> impl Iterator<Item = String> {
        list.iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .into_iter()
    }

    #[test]
    fn seed_requires_a_database_url() {
        let err = parse(args(&["seed", "--scale", "s"])).expect_err("must reject a missing url");
        assert!(err.contains("--database-url"), "got: {err}");
    }

    #[test]
    fn seed_parses_every_flag() {
        let command = parse(args(&[
            "seed",
            "--database-url",
            "postgres://localhost/bench",
            "--scale",
            "m",
            "--history-days",
            "30",
            "--seed",
            "7",
            "--append",
        ]))
        .expect("all flags are valid");

        match command {
            Command::Seed {
                database_url,
                scale,
                history_days,
                seed,
                append,
            } => {
                assert_eq!(database_url, "postgres://localhost/bench");
                assert_eq!(scale, Scale::M);
                assert_eq!(history_days, 30);
                assert_eq!(seed, 7);
                assert!(append);
            }
            other => panic!("expected Seed, got {other:?}"),
        }
    }

    #[test]
    fn unknown_scale_is_rejected() {
        let err = parse(args(&[
            "seed",
            "--database-url",
            "postgres://localhost/bench",
            "--scale",
            "xxl",
        ]))
        .expect_err("xxl is not a scale");
        assert!(err.contains("xxl"), "got: {err}");
    }

    #[test]
    fn unknown_flags_are_rejected_rather_than_ignored() {
        let err = parse(args(&[
            "run",
            "--database-url",
            "postgres://localhost/bench",
            "--out",
            "/tmp/x",
            "--turbo",
        ]))
        .expect_err("a typo must not silently do nothing");
        assert!(err.contains("--turbo"), "got: {err}");
    }

    #[test]
    fn run_splits_the_only_list_on_commas() {
        let command = parse(args(&[
            "run",
            "--database-url",
            "postgres://localhost/bench",
            "--out",
            "/tmp/x",
            "--only",
            "tree.view_markers, cluster.statistics",
        ]))
        .expect("the flags are valid");

        match command {
            Command::Run { only, .. } => {
                assert_eq!(
                    only,
                    Some(vec![
                        "tree.view_markers".to_string(),
                        "cluster.statistics".to_string()
                    ])
                );
            }
            other => panic!("expected Run, got {other:?}"),
        }
    }

    #[test]
    fn campaign_falls_back_to_the_defaults() {
        let command = parse(args(&[
            "campaign",
            "--database-url",
            "postgres://localhost/bench",
            "--out",
            "/tmp/x",
        ]))
        .expect("the flags are valid");

        match command {
            Command::Campaign {
                history_days, seed, ..
            } => {
                assert_eq!(history_days, DEFAULT_HISTORY_DAYS);
                assert_eq!(seed, DEFAULT_SEED);
            }
            other => panic!("expected Campaign, got {other:?}"),
        }
    }
}
