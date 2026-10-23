use chrono::{DateTime, Utc};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use crate::{
    supervisor::UnitStatus,
    theme::{self, Tone},
};

pub(crate) fn tone_color(tone: Tone) -> Color {
    match tone {
        Tone::Good => Color::Green,
        Tone::Busy => Color::Yellow,
        Tone::Bad => Color::Red,
        Tone::Idle => Color::DarkGray,
    }
}

/// The unit row of the TUI's list; `up --detach` draws the same rows inline.
pub(crate) fn unit_line(
    unit: &UnitStatus,
    marker: Span<'static>,
    now: DateTime<Utc>,
) -> Line<'static> {
    let tone = theme::tone(&unit.state);
    let color = Style::new().fg(tone_color(tone));
    Line::from(vec![
        Span::styled(theme::symbol(tone), color),
        Span::raw(format!(" {:<16}", unit.name)),
        marker,
        Span::raw(" "),
        Span::styled(unit.state.short(), color),
        Span::styled(
            format!(" {}", theme::ago(unit.since, now)),
            Style::new().fg(Color::DarkGray),
        ),
    ])
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::{
        supervisor::UnitState,
        units::{Origin, UnitKind},
    };

    #[test]
    fn a_unit_line_shows_symbol_name_state_and_age() {
        let unit = UnitStatus {
            name: "backend".into(),
            kind: UnitKind::Service,
            origin: Origin::Local,
            state: UnitState::Ready,
            since: Utc::now(),
        };
        let text: String = unit_line(&unit, Span::raw("  "), Utc::now())
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect();
        assert!(text.starts_with("● backend"), "{text}");
        assert!(text.contains("ready 0s"), "{text}");
    }
}
