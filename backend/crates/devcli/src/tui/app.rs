use std::collections::VecDeque;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    ipc::Call,
    logs::LogRecord,
    supervisor::{UnitState, UnitStatus},
};

pub const LOG_CAPACITY: usize = 5_000;
const PAGE: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogScope {
    All,
    Selected,
}

#[derive(Debug)]
pub enum Msg {
    Key(KeyEvent),
    State(UnitStatus),
    Log(LogRecord),
    Flash(String),
    Tick,
    Disconnected,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Effect {
    Call(Call),
    Quit,
}

pub struct App {
    pub units: Vec<UnitStatus>,
    pub selected: usize,
    pub scope: LogScope,
    pub logs: VecDeque<LogRecord>,
    /// Lines between the bottom of the view and the newest matching line; 0 follows.
    pub scroll: usize,
    pub hosted: bool,
    pub flash: Option<String>,
    pub show_help: bool,
    pub disconnected: bool,
}

impl App {
    pub fn new(hosted: bool) -> Self {
        Self {
            units: Vec::new(),
            selected: 0,
            scope: LogScope::All,
            logs: VecDeque::new(),
            scroll: 0,
            hosted,
            flash: None,
            show_help: false,
            disconnected: false,
        }
    }

    pub fn update(&mut self, msg: Msg) -> Option<Effect> {
        match msg {
            Msg::Key(key) => return self.on_key(key),
            Msg::State(status) => match self.units.iter_mut().find(|u| u.name == status.name) {
                Some(unit) => *unit = status,
                None => self.units.push(status),
            },
            Msg::Log(record) => {
                if self.logs.len() == LOG_CAPACITY {
                    self.logs.pop_front();
                }
                if self.scroll > 0 && self.matches(&record) {
                    self.scroll += 1;
                }
                self.logs.push_back(record);
            }
            Msg::Flash(text) => self.flash = Some(text),
            Msg::Tick => {}
            Msg::Disconnected => {
                self.disconnected = true;
                self.flash = Some("the supervisor has stopped".to_owned());
            }
        }
        None
    }

    fn on_key(&mut self, key: KeyEvent) -> Option<Effect> {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Char('q') => return Some(Effect::Quit),
            KeyCode::Char('c') if ctrl => return Some(Effect::Quit),
            KeyCode::Down | KeyCode::Char('j') => self.select(true),
            KeyCode::Up | KeyCode::Char('k') => self.select(false),
            KeyCode::Tab => {
                self.scope = match self.scope {
                    LogScope::All => LogScope::Selected,
                    LogScope::Selected => LogScope::All,
                };
                self.scroll = 0;
            }
            KeyCode::PageUp => self.scroll = self.scroll.saturating_add(PAGE),
            KeyCode::PageDown => self.scroll = self.scroll.saturating_sub(PAGE),
            KeyCode::End | KeyCode::Char('G') => self.scroll = 0,
            KeyCode::Char('?') => self.show_help = !self.show_help,
            KeyCode::Char('r') => {
                return self.call(|unit| Call::Restart {
                    unit,
                    cascade: false,
                });
            }
            KeyCode::Char('R') => {
                return self.call(|unit| Call::Restart {
                    unit,
                    cascade: true,
                });
            }
            KeyCode::Char('s') => {
                let stopped = self.selected_unit()?.state == UnitState::Stopped;
                return self.call(|unit| {
                    if stopped {
                        Call::Start { unit }
                    } else {
                        Call::Stop { unit }
                    }
                });
            }
            _ => {}
        }
        None
    }

    fn call(&self, build: impl FnOnce(String) -> Call) -> Option<Effect> {
        self.selected_unit()
            .map(|unit| Effect::Call(build(unit.name.clone())))
    }

    fn select(&mut self, down: bool) {
        let Some(last) = self.units.len().checked_sub(1) else {
            return;
        };
        self.selected = if down {
            (self.selected + 1).min(last)
        } else {
            self.selected.saturating_sub(1)
        };
        if self.scope == LogScope::Selected {
            self.scroll = 0;
        }
    }

    pub fn selected_unit(&self) -> Option<&UnitStatus> {
        self.units.get(self.selected)
    }

    pub fn matches(&self, record: &LogRecord) -> bool {
        match self.scope {
            LogScope::All => true,
            LogScope::Selected => self
                .selected_unit()
                .is_some_and(|unit| unit.name == record.unit),
        }
    }

    pub fn visible_logs(&self, height: usize) -> Vec<&LogRecord> {
        let matching: Vec<&LogRecord> = self.logs.iter().filter(|r| self.matches(r)).collect();
        let end = matching.len().saturating_sub(self.scroll);
        let start = end.saturating_sub(height);
        matching[start..end].to_vec()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use chrono::Utc;

    use super::*;
    use crate::{
        config::LogFormat,
        logs::parse_line,
        units::{Origin, UnitKind},
    };

    pub(crate) fn status(name: &str, state: UnitState) -> UnitStatus {
        UnitStatus {
            name: name.into(),
            kind: UnitKind::Service,
            origin: Origin::Local,
            state,
            since: Utc::now(),
        }
    }

    pub(crate) fn record(unit: &str, message: &str) -> LogRecord {
        parse_line(unit, LogFormat::Plain, message, Utc::now())
    }

    fn key(code: KeyCode) -> Msg {
        Msg::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn app_with_two_units() -> App {
        let mut app = App::new(true);
        app.update(Msg::State(status("db", UnitState::Ready)));
        app.update(Msg::State(status("backend", UnitState::Stopped)));
        app
    }

    #[test]
    fn state_updates_replace_known_units_in_place() {
        let mut app = app_with_two_units();
        app.update(Msg::State(status(
            "db",
            UnitState::Failed { reason: "x".into() },
        )));

        let names: Vec<&str> = app.units.iter().map(|u| u.name.as_str()).collect();
        assert_eq!(names, ["db", "backend"]);
        assert!(matches!(app.units[0].state, UnitState::Failed { .. }));
    }

    #[test]
    fn selection_stays_within_the_unit_list() {
        let mut app = app_with_two_units();
        app.update(key(KeyCode::Up));
        assert_eq!(app.selected, 0);
        app.update(key(KeyCode::Char('j')));
        app.update(key(KeyCode::Char('j')));
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn keys_turn_into_supervisor_calls() {
        let mut app = app_with_two_units();
        assert_eq!(
            app.update(key(KeyCode::Char('r'))),
            Some(Effect::Call(Call::Restart {
                unit: "db".into(),
                cascade: false
            }))
        );
        assert_eq!(
            app.update(key(KeyCode::Char('R'))),
            Some(Effect::Call(Call::Restart {
                unit: "db".into(),
                cascade: true
            }))
        );
        assert_eq!(
            app.update(key(KeyCode::Char('s'))),
            Some(Effect::Call(Call::Stop { unit: "db".into() }))
        );
        app.update(key(KeyCode::Down));
        assert_eq!(
            app.update(key(KeyCode::Char('s'))),
            Some(Effect::Call(Call::Start {
                unit: "backend".into()
            }))
        );
        assert_eq!(app.update(key(KeyCode::Char('q'))), Some(Effect::Quit));
        assert_eq!(
            app.update(Msg::Key(KeyEvent::new(
                KeyCode::Char('c'),
                KeyModifiers::CONTROL
            ))),
            Some(Effect::Quit)
        );
    }

    #[test]
    fn tab_switches_between_all_and_the_selected_unit() {
        let mut app = app_with_two_units();
        app.update(Msg::Log(record("db", "from db")));
        app.update(Msg::Log(record("backend", "from backend")));
        assert_eq!(app.visible_logs(10).len(), 2);

        app.update(key(KeyCode::Tab));
        assert_eq!(app.scope, LogScope::Selected);
        let visible: Vec<&str> = app
            .visible_logs(10)
            .iter()
            .map(|r| r.message.as_str())
            .collect();
        assert_eq!(visible, ["from db"]);
    }

    #[test]
    fn a_scrolled_view_stays_put_when_new_lines_arrive() {
        let mut app = app_with_two_units();
        for i in 0..30 {
            app.update(Msg::Log(record("db", &i.to_string())));
        }
        app.update(key(KeyCode::PageUp));
        let before: Vec<String> = app
            .visible_logs(5)
            .iter()
            .map(|r| r.message.clone())
            .collect();

        app.update(Msg::Log(record("db", "new")));

        let after: Vec<String> = app
            .visible_logs(5)
            .iter()
            .map(|r| r.message.clone())
            .collect();
        assert_eq!(before, after);
        app.update(key(KeyCode::Char('G')));
        assert_eq!(app.visible_logs(1)[0].message, "new");
    }

    #[test]
    fn the_log_buffer_is_bounded() {
        let mut app = App::new(false);
        for i in 0..=LOG_CAPACITY {
            app.update(Msg::Log(record("db", &i.to_string())));
        }
        assert_eq!(app.logs.len(), LOG_CAPACITY);
        assert_eq!(app.logs[0].message, "1");
    }
}
