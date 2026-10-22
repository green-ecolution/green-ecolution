use std::io::{self, Write};

use anstyle::{AnsiColor, Style};
use chrono::{DateTime, Local, Utc};

use crate::{
    logs::{Level, LogRecord},
    supervisor::UnitStatus,
    theme::{self, Tone},
    units::Origin,
};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::LogFormat,
        logs::{parse_line, strip_ansi},
        supervisor::UnitState,
        units::UnitKind,
    };

    fn rendered(write: impl FnOnce(&mut Vec<u8>) -> io::Result<()>) -> String {
        let mut buf = Vec::new();
        write(&mut buf).unwrap();
        strip_ansi(&String::from_utf8(buf).unwrap())
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
}
