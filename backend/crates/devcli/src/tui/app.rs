use std::collections::VecDeque;

use chrono::Utc;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::input::{InputOutcome, LineInput};
use crate::{
    ipc::Call,
    logs::{Filter, LogRecord},
    supervisor::{UnitState, UnitStatus},
};

pub const LOG_CAPACITY: usize = 5_000;
const PAGE: usize = 20;
const HALF_PAGE: usize = PAGE / 2;
const WHEEL_STEP: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Units,
    Logs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogScope {
    All,
    Selected,
}

#[derive(Debug)]
pub enum Msg {
    Key(KeyEvent),
    /// A mouse wheel notch over `pane`; it scrolls that pane without focusing it.
    Scroll {
        pane: Pane,
        down: bool,
    },
    Focus(Pane),
    State(UnitStatus),
    Log(LogRecord),
    Flash(String),
    Tick,
    Disconnected,
    /// The supervisor this TUI hosts was told to shut down.
    HostStopped,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Effect {
    Call(Call),
    Quit,
}

pub struct App {
    pub units: Vec<UnitStatus>,
    pub selected: usize,
    pub focus: Pane,
    pub scope: LogScope,
    pub filter: Filter,
    pub filter_text: String,
    pub input: Option<LineInput>,
    pub filter_error: Option<String>,
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
            focus: Pane::Units,
            scope: LogScope::All,
            filter: Filter::default(),
            filter_text: String::new(),
            input: None,
            filter_error: None,
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
            Msg::Scroll { pane, down } => self.step(pane, down, WHEEL_STEP),
            Msg::Focus(pane) => self.focus = pane,
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
                // max_scroll() walks the whole buffer; a following view needs no clamping.
                if self.scroll > 0 {
                    self.scroll = self.scroll.min(self.max_scroll());
                }
            }
            Msg::Flash(text) => self.flash = Some(text),
            Msg::Tick => {}
            Msg::Disconnected => {
                self.disconnected = true;
                self.flash = Some("the supervisor has stopped".to_owned());
            }
            Msg::HostStopped => return Some(Effect::Quit),
        }
        None
    }

    fn on_key(&mut self, key: KeyEvent) -> Option<Effect> {
        if let Some(input) = &mut self.input {
            self.filter_error = None;
            match input.handle(key) {
                InputOutcome::Editing => {}
                InputOutcome::Cancel => self.input = None,
                InputOutcome::Submit(text) => self.apply_filter(&text),
            }
            return None;
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Char('q') => return Some(Effect::Quit),
            KeyCode::Char('c') if ctrl => return Some(Effect::Quit),
            KeyCode::Char('d') if ctrl => self.scroll_down(HALF_PAGE),
            KeyCode::Char('u') if ctrl => self.scroll_up(HALF_PAGE),
            KeyCode::Left | KeyCode::Char('h' | '[') => self.focus = Pane::Units,
            KeyCode::Right | KeyCode::Char('l' | ']') => self.focus = Pane::Logs,
            KeyCode::Down | KeyCode::Char('j') => self.step(self.focus, true, 1),
            KeyCode::Up | KeyCode::Char('k') => self.step(self.focus, false, 1),
            KeyCode::Tab => {
                self.scope = match self.scope {
                    LogScope::All => LogScope::Selected,
                    LogScope::Selected => LogScope::All,
                };
                self.scroll = 0;
            }
            KeyCode::Enter if self.focus == Pane::Units => {
                self.scope = LogScope::Selected;
                self.scroll = 0;
            }
            KeyCode::PageUp => self.scroll_up(PAGE),
            KeyCode::PageDown => self.scroll_down(PAGE),
            KeyCode::Home | KeyCode::Char('g') => self.scroll = self.max_scroll(),
            KeyCode::End | KeyCode::Char('G') => self.scroll = 0,
            KeyCode::Char('?') => self.show_help = !self.show_help,
            KeyCode::Char('/') => self.input = Some(LineInput::new(&self.filter_text)),
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

    fn step(&mut self, pane: Pane, down: bool, lines: usize) {
        match pane {
            Pane::Units => self.select(down),
            Pane::Logs if down => self.scroll_down(lines),
            Pane::Logs => self.scroll_up(lines),
        }
    }

    fn scroll_up(&mut self, lines: usize) {
        self.scroll = self.scroll.saturating_add(lines).min(self.max_scroll());
    }

    fn scroll_down(&mut self, lines: usize) {
        self.scroll = self.scroll.saturating_sub(lines);
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

    fn apply_filter(&mut self, text: &str) {
        match text.parse::<Filter>() {
            Ok(filter) => {
                self.filter = filter;
                self.filter_text = text.trim().to_owned();
                self.input = None;
                self.scroll = 0;
            }
            Err(err) => self.filter_error = Some(format!("invalid filter: {err}")),
        }
    }

    pub fn matches(&self, record: &LogRecord) -> bool {
        let in_scope = match self.scope {
            LogScope::All => true,
            LogScope::Selected => self
                .selected_unit()
                .is_some_and(|unit| unit.name == record.unit),
        };
        in_scope && self.filter.matches(record, Utc::now())
    }

    fn max_scroll(&self) -> usize {
        self.logs
            .iter()
            .filter(|r| self.matches(r))
            .count()
            .saturating_sub(1)
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

    fn ctrl(code: KeyCode) -> Msg {
        Msg::Key(KeyEvent::new(code, KeyModifiers::CONTROL))
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
        app.update(key(KeyCode::Char('l')));
        app.update(key(KeyCode::Char('k')));
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
    fn scrolling_stops_at_the_oldest_line() {
        let mut app = app_with_two_units();
        for i in 0..3 {
            app.update(Msg::Log(record("db", &i.to_string())));
        }
        app.update(key(KeyCode::Char(']')));
        for _ in 0..10 {
            app.update(key(KeyCode::Up));
        }
        assert_eq!(app.visible_logs(1)[0].message, "0");
        app.update(Msg::Log(record("db", "3")));
        assert_eq!(app.visible_logs(1)[0].message, "0");

        app.update(ctrl(KeyCode::Char('d')));
        assert_eq!(app.scroll, 0);
        app.update(key(KeyCode::Char('g')));
        assert_eq!(app.visible_logs(1)[0].message, "0");
        app.update(key(KeyCode::Char('G')));
        assert_eq!(app.visible_logs(1)[0].message, "3");
    }

    #[test]
    fn the_focused_pane_decides_what_j_and_k_move() {
        let mut app = app_with_two_units();
        for i in 0..5 {
            app.update(Msg::Log(record("db", &i.to_string())));
        }
        assert_eq!(app.focus, Pane::Units);

        app.update(key(KeyCode::Char('j')));
        assert_eq!((app.selected, app.scroll), (1, 0));

        app.update(key(KeyCode::Right));
        assert_eq!(app.focus, Pane::Logs);
        app.update(key(KeyCode::Char('k')));
        app.update(key(KeyCode::Char('k')));
        app.update(key(KeyCode::Char('j')));
        assert_eq!((app.selected, app.scroll), (1, 1));

        app.update(key(KeyCode::Char('h')));
        assert_eq!(app.focus, Pane::Units);
        app.update(key(KeyCode::Char('[')));
        assert_eq!(app.focus, Pane::Units);
        app.update(key(KeyCode::Char('k')));
        assert_eq!((app.selected, app.scroll), (0, 1));
    }

    #[test]
    fn the_wheel_scrolls_the_pane_under_the_cursor_without_focusing_it() {
        let mut app = app_with_two_units();
        for i in 0..10 {
            app.update(Msg::Log(record("db", &i.to_string())));
        }

        app.update(Msg::Scroll {
            pane: Pane::Logs,
            down: false,
        });
        assert_eq!((app.focus, app.scroll), (Pane::Units, WHEEL_STEP));

        app.update(Msg::Scroll {
            pane: Pane::Units,
            down: true,
        });
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn enter_on_a_unit_shows_only_its_logs() {
        let mut app = app_with_two_units();
        app.update(key(KeyCode::Char('j')));
        app.update(key(KeyCode::Enter));
        assert_eq!(app.scope, LogScope::Selected);
        app.update(key(KeyCode::Enter));
        assert_eq!(app.scope, LogScope::Selected);

        app.update(key(KeyCode::Tab));
        app.update(key(KeyCode::Char('l')));
        app.update(key(KeyCode::Enter));
        assert_eq!(app.scope, LogScope::All);
    }

    #[test]
    fn a_click_focuses_the_pane() {
        let mut app = app_with_two_units();
        app.update(Msg::Focus(Pane::Logs));
        assert_eq!(app.focus, Pane::Logs);
        app.update(Msg::Focus(Pane::Units));
        assert_eq!(app.focus, Pane::Units);
    }

    #[test]
    fn a_stopped_host_quits_while_a_lost_connection_only_shows_it() {
        let mut app = app_with_two_units();
        assert_eq!(app.update(Msg::Disconnected), None);
        assert!(app.disconnected);

        assert_eq!(app.update(Msg::HostStopped), Some(Effect::Quit));
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

    fn type_text(app: &mut App, text: &str) {
        for c in text.chars() {
            app.update(key(KeyCode::Char(c)));
        }
    }

    #[test]
    fn slash_opens_a_filter_that_narrows_the_logs() {
        let mut app = app_with_two_units();
        app.update(Msg::Log(record("db", "ERROR broken")));
        app.update(Msg::Log(record("db", "fine")));

        app.update(key(KeyCode::Char('/')));
        type_text(&mut app, "level:error q");
        assert_eq!(
            app.update(key(KeyCode::Char('q'))),
            None,
            "keys go to the input"
        );
        app.update(key(KeyCode::Backspace));
        app.update(key(KeyCode::Backspace));
        app.update(key(KeyCode::Backspace));
        app.update(key(KeyCode::Enter));

        assert!(app.input.is_none());
        assert_eq!(app.filter_text, "level:error");
        let visible: Vec<&str> = app
            .visible_logs(10)
            .iter()
            .map(|r| r.message.as_str())
            .collect();
        assert_eq!(visible, ["ERROR broken"]);
    }

    #[test]
    fn an_invalid_filter_keeps_the_input_open() {
        let mut app = app_with_two_units();
        app.update(key(KeyCode::Char('/')));
        type_text(&mut app, "level:loud");
        app.update(key(KeyCode::Enter));

        assert!(app.input.is_some());
        assert!(
            app.filter_error
                .as_deref()
                .unwrap()
                .contains("unknown level `loud`")
        );
        assert_eq!(app.flash, None);
        assert!(app.filter.is_empty());
    }

    #[test]
    fn escape_leaves_the_filter_unchanged_and_an_empty_one_clears_it() {
        let mut app = app_with_two_units();
        app.update(key(KeyCode::Char('/')));
        type_text(&mut app, "unit:db");
        app.update(key(KeyCode::Enter));

        app.update(key(KeyCode::Char('/')));
        app.update(key(KeyCode::Backspace));
        app.update(key(KeyCode::Esc));
        assert_eq!(app.filter_text, "unit:db");

        app.update(key(KeyCode::Char('/')));
        app.update(ctrl(KeyCode::Char('u')));
        app.update(key(KeyCode::Enter));
        assert!(app.filter.is_empty());
        assert_eq!(app.filter_text, "");
    }
}
