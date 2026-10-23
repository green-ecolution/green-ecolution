use crossterm::event::{KeyCode, KeyEvent};

use super::app::Mode;

type Keys = &'static [(&'static str, &'static str)];

const LOGS: Keys = &[
    ("h/l ←/→", "focus the unit list or the logs"),
    ("j/k ↓/↑", "select a unit or scroll the logs"),
    ("^u/^d", "half page up/down"),
    ("PgUp/PgDn", "page up/down"),
    ("g/G", "oldest line / follow"),
    ("Enter", "show the selected unit's logs"),
    ("Tab", "all units / selected unit"),
    ("/", "filter the logs"),
    ("r", "restart the unit"),
    ("R", "restart with dependents"),
    ("s", "start/stop (dismisses a failed task)"),
    ("f", "run the unit's suggested fix"),
    ("t", "requests"),
    ("d", "doctor"),
];

const REQUESTS: Keys = &[
    ("h/l ←/→", "focus the request list or the trace"),
    ("j/k ↓/↑", "select a request or scroll the trace"),
    ("Enter", "open the trace"),
    ("o", "order by newest/slowest"),
    ("^u/^d", "half page up/down"),
    ("PgUp/PgDn", "page up/down"),
    ("g/G", "top/bottom of the trace"),
    ("d", "doctor"),
    ("t/Esc", "back to the logs"),
];

const DOCTOR: Keys = &[
    ("j/k ↓/↑", "select a finding"),
    ("g/G", "first/last finding"),
    ("f", "run the suggested fix"),
    ("r", "check again"),
    ("R", "deep check"),
    ("t", "requests"),
    ("d/Esc", "back to the logs"),
];

const GLOBAL: Keys = &[
    ("p", "switch profile"),
    (": ^k", "command palette"),
    ("?", "this help"),
    ("q ^c", "quit"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpLine {
    Heading(&'static str),
    Key(&'static str, &'static str),
    Blank,
}

pub fn lines(mode: Mode) -> Vec<HelpLine> {
    let (title, keys) = match mode {
        Mode::Logs => ("Logs", LOGS),
        Mode::Requests => ("Requests", REQUESTS),
        Mode::Doctor => ("Doctor", DOCTOR),
    };
    let mut lines = vec![HelpLine::Heading(title)];
    lines.extend(keys.iter().map(|&(key, what)| HelpLine::Key(key, what)));
    lines.push(HelpLine::Blank);
    lines.push(HelpLine::Heading("Everywhere"));
    lines.extend(GLOBAL.iter().map(|&(key, what)| HelpLine::Key(key, what)));
    lines
}

#[derive(Debug, PartialEq, Eq)]
pub enum HelpOutcome {
    Open,
    Closed,
}

pub struct Help {
    lines: Vec<HelpLine>,
    scroll: usize,
}

impl Help {
    pub fn new(mode: Mode) -> Self {
        Self {
            lines: lines(mode),
            scroll: 0,
        }
    }

    pub fn lines(&self) -> &[HelpLine] {
        &self.lines
    }

    /// The first line to show; the view clamps it further so the box never ends half empty.
    pub fn scroll(&self) -> usize {
        self.scroll
    }

    pub fn handle(&mut self, key: KeyEvent) -> HelpOutcome {
        let last = self.lines.len().saturating_sub(1);
        match key.code {
            KeyCode::Esc | KeyCode::Char('?' | 'q') => return HelpOutcome::Closed,
            KeyCode::Down | KeyCode::Char('j') => self.scroll = (self.scroll + 1).min(last),
            KeyCode::Up | KeyCode::Char('k') => self.scroll = self.scroll.saturating_sub(1),
            KeyCode::Home | KeyCode::Char('g') => self.scroll = 0,
            KeyCode::End | KeyCode::Char('G') => self.scroll = last,
            _ => {}
        }
        HelpOutcome::Open
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::KeyModifiers;

    use super::*;

    fn press(help: &mut Help, code: KeyCode) -> HelpOutcome {
        help.handle(KeyEvent::new(code, KeyModifiers::NONE))
    }

    #[test]
    fn every_mode_lists_its_own_keys_and_the_global_ones() {
        for (mode, heading) in [
            (Mode::Logs, "Logs"),
            (Mode::Requests, "Requests"),
            (Mode::Doctor, "Doctor"),
        ] {
            let lines = lines(mode);
            assert_eq!(lines[0], HelpLine::Heading(heading));
            assert!(lines.contains(&HelpLine::Heading("Everywhere")));
            assert!(lines.contains(&HelpLine::Key("q ^c", "quit")));
        }
    }

    #[test]
    fn scrolling_stays_within_the_lines_and_close_keys_close() {
        let mut help = Help::new(Mode::Doctor);
        let last = help.lines().len() - 1;
        press(&mut help, KeyCode::Char('k'));
        assert_eq!(help.scroll(), 0);
        press(&mut help, KeyCode::Char('G'));
        assert_eq!(help.scroll(), last);
        press(&mut help, KeyCode::Char('j'));
        assert_eq!(help.scroll(), last);
        assert_eq!(press(&mut help, KeyCode::Char('r')), HelpOutcome::Open);
        for code in [KeyCode::Esc, KeyCode::Char('?'), KeyCode::Char('q')] {
            assert_eq!(press(&mut help, code), HelpOutcome::Closed);
        }
    }
}
