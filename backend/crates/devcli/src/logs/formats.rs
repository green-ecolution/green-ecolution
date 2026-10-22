use chrono::{DateTime, NaiveDateTime, Utc};
use serde_json::{Map, Value};

use super::Level;

pub(super) struct Recognized {
    pub timestamp: Option<DateTime<Utc>>,
    pub level: Level,
    pub target: Option<String>,
    pub message: String,
    pub fields: Map<String, Value>,
    /// Belongs to the previous record (a Postgres DETAIL, a rustc code frame) and inherits its level.
    pub continuation: bool,
}

/// `line` is already free of ANSI sequences.
pub(super) fn recognize(line: &str) -> Option<Recognized> {
    postgres(line)
        .or_else(|| quarkus(line))
        .or_else(|| traefik(line))
        .or_else(|| cargo(line))
}

fn field(key: &str, value: &str) -> Map<String, Value> {
    Map::from_iter([(key.to_owned(), Value::String(value.to_owned()))])
}

/// `2026-10-22 12:17:29.490 UTC [1] LOG:  database system is ready to accept connections`
fn postgres(line: &str) -> Option<Recognized> {
    let (stamp, rest) = line.split_once(" UTC [")?;
    let timestamp = NaiveDateTime::parse_from_str(stamp, "%Y-%m-%d %H:%M:%S%.f")
        .ok()?
        .and_utc();
    let (pid, rest) = rest.split_once("] ")?;
    let (severity, message) = rest.split_once(':')?;
    let message = message.trim_start();
    let (level, continuation) = match severity {
        "LOG" | "INFO" | "NOTICE" => (Level::Info, false),
        "WARNING" => (Level::Warn, false),
        "ERROR" | "FATAL" | "PANIC" => (Level::Error, false),
        "DETAIL" | "HINT" | "QUERY" | "CONTEXT" | "LOCATION" | "STATEMENT" => {
            (Level::Unknown, true)
        }
        debug if debug.starts_with("DEBUG") => (Level::Debug, false),
        _ => return None,
    };
    Some(Recognized {
        timestamp: Some(timestamp),
        level,
        target: None,
        message: if continuation {
            format!("{severity}: {message}")
        } else {
            message.to_owned()
        },
        fields: field("pid", pid),
        continuation,
    })
}

/// `2026-10-22 12:17:37,808 WARN  [org.keycloak.common.Profile] (main) Deprecated features …`;
/// the stamp carries no zone, and the Keycloak image logs in UTC.
fn quarkus(line: &str) -> Option<Recognized> {
    let timestamp = NaiveDateTime::parse_from_str(line.get(..23)?, "%Y-%m-%d %H:%M:%S,%3f")
        .ok()?
        .and_utc();
    let (token, rest) = line.get(23..)?.trim_start().split_once(' ')?;
    let level = match token {
        "TRACE" => Level::Trace,
        "DEBUG" => Level::Debug,
        "INFO" => Level::Info,
        "WARN" | "WARNING" => Level::Warn,
        "ERROR" | "FATAL" | "SEVERE" => Level::Error,
        _ => return None,
    };
    let (category, rest) = rest.trim_start().strip_prefix('[')?.split_once("] ")?;
    let (fields, message) = match rest.strip_prefix('(').and_then(|r| r.split_once(") ")) {
        Some((thread, message)) => (field("thread", thread), message),
        None => (Map::new(), rest),
    };
    Some(Recognized {
        timestamp: Some(timestamp),
        level,
        target: Some(category.to_owned()),
        message: message.to_owned(),
        fields,
        continuation: false,
    })
}

/// Traefik v3: `2026-10-22T12:31:00Z ERR Cannot create service error="…" providerName=docker`
fn traefik(line: &str) -> Option<Recognized> {
    let (stamp, rest) = line.split_once(' ')?;
    let timestamp = DateTime::parse_from_rfc3339(stamp)
        .ok()?
        .with_timezone(&Utc);
    let (token, message) = rest.split_once(' ').unwrap_or((rest, ""));
    let level = match token {
        "TRC" => Level::Trace,
        "DBG" => Level::Debug,
        "INF" => Level::Info,
        "WRN" => Level::Warn,
        "ERR" | "FTL" | "PNC" => Level::Error,
        _ => return None,
    };
    Some(Recognized {
        timestamp: Some(timestamp),
        level,
        target: None,
        message: message.to_owned(),
        fields: Map::new(),
        continuation: false,
    })
}

const CARGO_STEPS: [&str; 9] = [
    "Compiling",
    "Checking",
    "Finished",
    "Running",
    "Blocking",
    "Updating",
    "Downloading",
    "Downloaded",
    "Locking",
];

/// Cargo right-aligns its progress verbs; rustc diagnostics start at column 0 and are followed
/// by indented location and code-frame lines.
fn cargo(line: &str) -> Option<Recognized> {
    let trimmed = line.trim_start();
    let rustc = |level, message: &str, fields, continuation| Recognized {
        timestamp: None,
        level,
        target: Some("rustc".to_owned()),
        message: message.to_owned(),
        fields,
        continuation,
    };
    if trimmed.len() < line.len() {
        if let Some(location) = trimmed.strip_prefix("--> ") {
            return Some(rustc(
                Level::Unknown,
                trimmed,
                field("file", location),
                true,
            ));
        }
        if is_code_frame(trimmed) {
            return Some(rustc(Level::Unknown, line, Map::new(), true));
        }
        let (step, _) = trimmed.split_once(' ')?;
        return CARGO_STEPS.contains(&step).then(|| Recognized {
            timestamp: None,
            level: Level::Info,
            target: Some("cargo".to_owned()),
            message: trimmed.to_owned(),
            fields: Map::new(),
            continuation: false,
        });
    }
    for (prefix, level) in [("error", Level::Error), ("warning", Level::Warn)] {
        let Some(rest) = line.strip_prefix(prefix) else {
            continue;
        };
        let code = rest.strip_prefix('[').and_then(|r| r.split_once("]:"));
        if let Some((code, _)) = code {
            return Some(rustc(level, line, field("code", code), false));
        }
        if rest.starts_with(':') {
            return Some(rustc(level, line, Map::new(), false));
        }
    }
    None
}

/// `  |     let y = x;`, `3 |     let y = x;`, `  = note: …`
fn is_code_frame(trimmed: &str) -> bool {
    let rest = trimmed
        .trim_start_matches(|c: char| c.is_ascii_digit())
        .trim_start();
    rest.starts_with('|') || rest.starts_with("= ")
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use crate::{
        config::LogFormat,
        logs::{Level, LineParser, parse_line},
    };

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-10-22T13:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn plain(line: &str) -> crate::logs::LogRecord {
        parse_line("svc", LogFormat::Plain, line, now())
    }

    #[test]
    fn reads_postgres_lines() {
        let record = plain(
            "2026-10-22 12:17:29.508 UTC [1] LOG:  database system is ready to accept connections",
        );
        assert_eq!(record.level, Level::Info);
        assert_eq!(
            record.message,
            "database system is ready to accept connections"
        );
        assert_eq!(
            record.timestamp.to_rfc3339(),
            "2026-10-22T12:17:29.508+00:00"
        );
        assert_eq!(record.fields["pid"], "1");

        let error = plain(
            r#"2026-10-22 12:30:01.002 UTC [88] ERROR:  column "foo" does not exist at character 8"#,
        );
        assert_eq!(error.level, Level::Error);
        assert_eq!(
            error.message,
            r#"column "foo" does not exist at character 8"#
        );
    }

    #[test]
    fn reads_keycloak_lines() {
        let record = plain(
            "2026-10-22 12:17:39,662 WARN  [org.keycloak.common.Profile] (main) Deprecated features identity-brokering-api:v1 enabled by default.",
        );
        assert_eq!(record.level, Level::Warn);
        assert_eq!(
            record.target.as_deref(),
            Some("org.keycloak.common.Profile")
        );
        assert_eq!(record.fields["thread"], "main");
        assert_eq!(
            record.message,
            "Deprecated features identity-brokering-api:v1 enabled by default."
        );
        assert_eq!(
            record.timestamp.to_rfc3339(),
            "2026-10-22T12:17:39.662+00:00"
        );

        let warning = plain(
            "2026-10-22 12:17:39,613 WARNING [io.opentelemetry.usage] (main) OpenTelemetry API usage issue detected.",
        );
        assert_eq!(warning.level, Level::Warn);
    }

    #[test]
    fn reads_traefik_lines() {
        let record = plain(
            r#"2026-10-22T12:31:00Z ERR Cannot create service error="subset not found" providerName=docker"#,
        );
        assert_eq!(record.level, Level::Error);
        assert_eq!(
            record.message,
            r#"Cannot create service error="subset not found" providerName=docker"#
        );
        assert_eq!(record.timestamp.to_rfc3339(), "2026-10-22T12:31:00+00:00");
    }

    #[test]
    fn reads_cargo_and_rustc_output() {
        let header = plain(
            "\u{1b}[1m\u{1b}[91merror[E0425]\u{1b}[0m\u{1b}[1m: cannot find value `x` in this scope\u{1b}[0m",
        );
        assert_eq!(header.level, Level::Error);
        assert_eq!(header.target.as_deref(), Some("rustc"));
        assert_eq!(header.fields["code"], "E0425");
        assert_eq!(
            header.message,
            "error[E0425]: cannot find value `x` in this scope"
        );

        let warning = plain("warning: unused variable: `y`");
        assert_eq!(warning.level, Level::Warn);
        assert_eq!(warning.target.as_deref(), Some("rustc"));

        let step = plain(
            "\u{1b}[1m\u{1b}[92m   Compiling\u{1b}[0m server v0.8.0 (/repo/backend/crates/server)",
        );
        assert_eq!(step.level, Level::Info);
        assert_eq!(step.target.as_deref(), Some("cargo"));
        assert_eq!(
            step.message,
            "Compiling server v0.8.0 (/repo/backend/crates/server)"
        );
    }

    #[test]
    fn prose_that_only_looks_like_cargo_stays_generic() {
        let record = plain(
            "Running the server in development mode. DO NOT use this configuration in production.",
        );
        assert_eq!(record.target, None);
    }

    #[test]
    fn continuation_lines_inherit_the_level_of_their_header() {
        let mut parser = LineParser::new("backend", LogFormat::TracingJson);
        parser.parse("error[E0425]: cannot find value `x` in this scope", now());
        let location = parser.parse(" --> crates/server/src/lib.rs:3:5", now());
        let frame = parser.parse("  |     let y = x;", now());

        assert_eq!(location.level, Level::Error);
        assert_eq!(location.fields["file"], "crates/server/src/lib.rs:3:5");
        assert_eq!(location.target.as_deref(), Some("rustc"));
        assert_eq!(frame.level, Level::Error);

        let mut db = LineParser::new("db", LogFormat::Plain);
        db.parse(r#"2026-10-22 12:30:01.002 UTC [88] ERROR:  duplicate key value violates unique constraint "trees_pkey""#, now());
        let detail = db.parse(
            "2026-10-22 12:30:01.002 UTC [88] DETAIL:  Key (id)=(1) already exists.",
            now(),
        );
        assert_eq!(detail.level, Level::Error);
        assert_eq!(detail.message, "DETAIL: Key (id)=(1) already exists.");
    }

    #[test]
    fn a_continuation_without_a_header_keeps_no_level() {
        let mut parser = LineParser::new("backend", LogFormat::Plain);
        assert_eq!(
            parser.parse(" --> src/main.rs:1:1", now()).level,
            Level::Unknown
        );
    }

    #[test]
    fn a_new_header_resets_the_inherited_level() {
        let mut parser = LineParser::new("backend", LogFormat::Plain);
        parser.parse("error: could not compile `server`", now());
        parser.parse("warning: unused import", now());
        assert_eq!(
            parser.parse(" --> src/lib.rs:1:5", now()).level,
            Level::Warn
        );
    }
}
