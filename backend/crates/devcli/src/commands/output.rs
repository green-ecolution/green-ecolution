use std::io::{self, Write};

use anstyle::{AnsiColor, Style};
use chrono::{DateTime, Local, Utc};

use crate::{
    doctor::{Report, Severity},
    logs::{
        Level, LogRecord,
        trace::{self, Trace, TraceLineKind},
    },
    profiles::ProfileList,
    supervisor::{ProfileChange, UnitStatus},
    theme::{self, Tone},
    units::Origin,
};

pub fn write_profile_list(out: &mut impl Write, list: &ProfileList) -> io::Result<()> {
    for profile in &list.available {
        let marker = if *profile == list.active { "* " } else { "  " };
        writeln!(out, "{marker}{profile}")?;
    }
    Ok(())
}

pub fn write_profile_change(out: &mut impl Write, change: &ProfileChange) -> io::Result<()> {
    writeln!(out, "switched to {}", change.profile)?;
    for (label, units) in [
        ("stopped", &change.stopped),
        ("restarted", &change.restarted),
        ("started", &change.started),
    ] {
        if !units.is_empty() {
            writeln!(out, "  {label}: {}", units.join(", "))?;
        }
    }
    for warning in &change.warnings {
        writeln!(out, "  warning: {warning}")?;
    }
    Ok(())
}

const UNIT_COLORS: [AnsiColor; theme::UNIT_SLOTS] = [
    AnsiColor::Cyan,
    AnsiColor::Magenta,
    AnsiColor::Blue,
    AnsiColor::Green,
    AnsiColor::Yellow,
    AnsiColor::BrightCyan,
];

fn fg(color: AnsiColor) -> Style {
    Style::new().fg_color(Some(color.into()))
}

fn dim() -> Style {
    Style::new().dimmed()
}

fn level_style(level: Level) -> Style {
    match level {
        Level::Error => fg(AnsiColor::Red).bold(),
        Level::Warn => fg(AnsiColor::Yellow),
        Level::Info => fg(AnsiColor::Green),
        Level::Debug => fg(AnsiColor::Blue),
        Level::Trace | Level::Unknown => dim(),
    }
}

fn tone_style(tone: Tone) -> Style {
    match tone {
        Tone::Good => fg(AnsiColor::Green),
        Tone::Busy => fg(AnsiColor::Yellow),
        Tone::Bad => fg(AnsiColor::Red).bold(),
        Tone::Idle => dim(),
    }
}

pub fn write_trace(out: &mut impl Write, trace: &Trace) -> io::Result<()> {
    let bold = Style::new().bold();
    match &trace.summary {
        Some(summary) => writeln!(
            out,
            "{bold}{} {} → {}{bold:#} in {} ms {dim}({}){dim:#}",
            summary.method,
            summary.path,
            summary.status,
            summary.latency_ms,
            trace.request_id,
            dim = dim(),
        )?,
        None => writeln!(
            out,
            "{bold}request {}{bold:#} {dim}(no response logged yet){dim:#}",
            trace.request_id,
            dim = dim(),
        )?,
    }
    for line in &trace.lines {
        write!(
            out,
            "{dim}{:>11}{dim:#} {}",
            trace::offset_label(line.offset_us),
            "  ".repeat(line.depth),
            dim = dim(),
        )?;
        match &line.kind {
            TraceLineKind::Enter { span } => writeln!(out, "▸ {span}")?,
            TraceLineKind::Event { level, message, .. } => {
                let style = level_style(*level);
                writeln!(out, "{style}{:<5}{style:#} {message}", level.label())?;
            }
            TraceLineKind::Close { target, busy, .. } => writeln!(
                out,
                "{dim}◂ {} busy {busy}{dim:#}",
                target.as_deref().unwrap_or("span"),
                dim = dim(),
            )?,
        }
    }
    Ok(())
}

pub fn write_log(out: &mut impl Write, record: &LogRecord) -> io::Result<()> {
    let time = record.timestamp.with_timezone(&Local).format("%H:%M:%S");
    let unit = fg(UNIT_COLORS[theme::unit_slot(&record.unit)]);
    let level = level_style(record.level);
    write!(
        out,
        "{dim}{time}{dim:#} {unit}{:<16}{unit:#} {level}{:<5}{level:#} {}",
        record.unit,
        record.level.label(),
        record.message,
        dim = dim(),
    )?;
    if let Some(request_id) = &record.request_id {
        write!(out, " {dim}req={request_id}{dim:#}", dim = dim())?;
    }
    writeln!(out)
}

pub fn write_state(out: &mut impl Write, status: &UnitStatus) -> io::Result<()> {
    let time = status.since.with_timezone(&Local).format("%H:%M:%S");
    let tone = theme::tone(&status.state);
    let style = tone_style(tone);
    writeln!(
        out,
        "{dim}{time}{dim:#} {style}{} {}: {}{style:#}",
        theme::symbol(tone),
        status.name,
        status.state,
        dim = dim(),
    )
}

pub fn write_status_table(
    out: &mut impl Write,
    units: &[UnitStatus],
    now: DateTime<Utc>,
) -> io::Result<()> {
    let width = units.iter().map(|u| u.name.len()).max().unwrap_or(0).max(4);
    writeln!(
        out,
        "  {:<width$}  {:<7}  {:<6}  STATE",
        "UNIT", "ORIGIN", "SINCE"
    )?;
    for unit in units {
        let tone = theme::tone(&unit.state);
        let style = tone_style(tone);
        let origin = match unit.origin {
            Origin::Local => "local",
            Origin::Compose => "compose",
            Origin::Task => "task",
        };
        writeln!(
            out,
            "{style}{}{style:#} {:<width$}  {origin:<7}  {:<6}  {style}{}{style:#}",
            theme::symbol(tone),
            unit.name,
            theme::ago(unit.since, now),
            unit.state,
        )?;
    }
    Ok(())
}

pub fn write_report(out: &mut impl Write, report: &Report) -> io::Result<()> {
    if report.findings.is_empty() && report.skipped.is_empty() {
        let good = fg(AnsiColor::Green);
        return writeln!(out, "{good}no findings{good:#}");
    }
    let width = report
        .findings
        .iter()
        .map(|f| f.id.len())
        .max()
        .unwrap_or(0);
    let indent = " ".repeat(width + 4);
    for finding in &report.findings {
        let (symbol, style) = match finding.severity {
            Severity::Error => ("✖", fg(AnsiColor::Red).bold()),
            Severity::Hint => ("●", fg(AnsiColor::Yellow)),
        };
        writeln!(
            out,
            "{style}{symbol}{style:#} {:<width$}  {}",
            finding.id, finding.title
        )?;
        for line in finding.detail.lines() {
            writeln!(out, "{indent}{dim}{line}{dim:#}", dim = dim())?;
        }
        if let Some(fix) = &finding.fix {
            let destructive = if fix.destructive {
                " (destructive)"
            } else {
                ""
            };
            writeln!(out, "{indent}fix: {}{destructive}", fix.command)?;
        }
    }
    for skipped in &report.skipped {
        writeln!(
            out,
            "{dim}skipped {}: {}{dim:#}",
            skipped.check,
            skipped.reason,
            dim = dim()
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::LogFormat,
        logs::{parse_line, strip_ansi, trace::Trace},
        supervisor::UnitState,
        units::UnitKind,
    };

    fn rendered(write: impl FnOnce(&mut Vec<u8>) -> io::Result<()>) -> String {
        let mut buf = Vec::new();
        write(&mut buf).unwrap();
        strip_ansi(&String::from_utf8(buf).unwrap())
    }

    #[test]
    fn traces_show_the_request_and_indent_by_span() {
        let records: Vec<LogRecord> = [
            r#"{"level":"DEBUG","message":"started processing request","request_id":"r1","spans":[{"name":"http_request","request_id":"r1"}],"target":"tower_http::trace::on_request","timestamp":"2026-10-22T12:26:16.372430Z"}"#,
            r#"{"level":"INFO","message":"close","request_id":"r1","spans":[{"name":"http_request","request_id":"r1"}],"target":"server::http::v1::info","time.busy":"33.5µs","time.idle":"8.07µs","timestamp":"2026-10-22T12:26:16.372675Z"}"#,
            r#"{"level":"DEBUG","message":"end of stream","request_id":"r1","spans":[{"http.request.method":"GET","http.response.status_code":200,"latency_ms":3,"name":"http_request","request_id":"r1","url.path":"/api/v1/info"}],"target":"tower_http::trace::on_eos","timestamp":"2026-10-22T12:26:16.372877Z"}"#,
        ]
        .iter()
        .map(|line| parse_line("backend", LogFormat::TracingJson, line, Utc::now()))
        .collect();

        let text = rendered(|out| write_trace(out, &Trace::build("r1", &records)));

        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines[0], "GET /api/v1/info → 200 in 3 ms (r1)");
        assert_eq!(lines[1], "   +0.000ms ▸ http_request");
        assert_eq!(lines[2], "   +0.000ms   DEBUG started processing request");
        assert_eq!(
            lines[3],
            "   +0.245ms   ◂ server::http::v1::info busy 33.5µs"
        );
    }

    #[test]
    fn log_lines_carry_unit_level_message_and_request() {
        let line = r#"{"timestamp":"2026-10-22T09:00:00Z","level":"WARN","message":"slow query","request_id":"req-7"}"#;
        let record = parse_line("backend", LogFormat::TracingJson, line, Utc::now());

        let text = rendered(|out| write_log(out, &record));

        assert!(text.contains("backend"), "{text}");
        assert!(text.contains("WARN"), "{text}");
        assert!(text.ends_with("slow query req=req-7\n"), "{text}");
    }

    #[test]
    fn the_status_table_lists_origin_age_and_state() {
        let now = Utc::now();
        let unit = UnitStatus {
            name: "backend".into(),
            kind: UnitKind::Service,
            origin: Origin::Local,
            state: UnitState::Waiting {
                on: vec!["migrations".into()],
            },
            since: now - chrono::Duration::seconds(42),
        };

        let text = rendered(|out| write_status_table(out, &[unit], now));

        assert!(text.contains("UNIT"), "{text}");
        assert!(
            text.contains("◐ backend  local    42s     waiting for migrations"),
            "{text}"
        );
    }

    #[test]
    fn the_status_table_names_tasks() {
        let task = UnitStatus {
            name: "migrate-up".into(),
            kind: UnitKind::Oneshot,
            origin: Origin::Task,
            state: UnitState::Starting,
            since: Utc::now(),
        };
        let text = rendered(|out| write_status_table(out, &[task], Utc::now()));
        assert!(text.contains("task"), "{text}");
    }
}

#[cfg(test)]
mod report_tests {
    use super::*;
    use crate::logs::strip_ansi;

    fn rendered(write: impl FnOnce(&mut Vec<u8>) -> io::Result<()>) -> String {
        let mut buf = Vec::new();
        write(&mut buf).unwrap();
        strip_ansi(&String::from_utf8(buf).unwrap())
    }

    #[test]
    fn profiles_mark_the_active_one_and_changes_list_what_happened() {
        use crate::{profiles::ProfileList, supervisor::ProfileChange};

        let list = ProfileList {
            active: "full".into(),
            available: vec!["full".into(), "minimal".into()],
        };
        assert_eq!(
            rendered(|out| write_profile_list(out, &list)),
            "* full\n  minimal\n"
        );

        let change = ProfileChange {
            profile: "prod-like".into(),
            stopped: vec!["pgadmin".into()],
            restarted: vec!["backend".into(), "frontend".into()],
            started: vec!["build-release".into()],
            warnings: vec!["Docker is not reachable".into()],
        };
        assert_eq!(
            rendered(|out| write_profile_change(out, &change)),
            "switched to prod-like\n  stopped: pgadmin\n  restarted: backend, frontend\n  started: build-release\n  warning: Docker is not reachable\n"
        );
    }

    #[test]
    fn the_report_lists_findings_fixes_and_skipped_checks() {
        use crate::doctor::{Finding, Fix, Report, Severity, Skipped};

        let report = Report {
            findings: vec![
                Finding::new(
                    "dev-db-migrations",
                    Severity::Error,
                    "dev database is 2 migration(s) behind",
                )
                .detail("Newest missing migration: 20260101.")
                .fix(Fix::just("migrate-up")),
                Finding::new(
                    "testcontainers",
                    Severity::Hint,
                    "3 testcontainer(s) left over",
                )
                .fix(Fix::just("test-clean").destructive()),
            ],
            skipped: vec![Skipped {
                check: "docker".into(),
                reason: "Docker is not reachable".into(),
            }],
        };

        let text = rendered(|out| write_report(out, &report));

        assert!(
            text.contains("✖ dev-db-migrations  dev database is 2 migration(s) behind"),
            "{text}"
        );
        assert!(
            text.contains("Newest missing migration: 20260101."),
            "{text}"
        );
        assert!(text.contains("fix: just migrate-up"), "{text}");
        assert!(
            text.contains("● testcontainers     3 testcontainer(s) left over"),
            "{text}"
        );
        assert!(
            text.contains("fix: just test-clean (destructive)"),
            "{text}"
        );
        assert!(
            text.contains("skipped docker: Docker is not reachable"),
            "{text}"
        );
        assert_eq!(
            rendered(|out| write_report(out, &Report::default())),
            "no findings\n"
        );
    }
}
