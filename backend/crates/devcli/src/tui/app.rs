use std::collections::VecDeque;

use chrono::Utc;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{
    input::{InputOutcome, LineInput},
    requests::Requests,
};
use crate::{
    doctor::Finding,
    ipc::Call,
    logs::{Filter, LogRecord, trace::Trace},
    supervisor::{UnitState, UnitStatus},
};

pub const LOG_CAPACITY: usize = 5_000;
const PAGE: usize = 20;
const HALF_PAGE: usize = PAGE / 2;
const WHEEL_STEP: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Logs,
    Requests,
    Doctor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Units,
    Logs,
    Requests,
    Trace,
    Findings,
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
    Trace(Trace),
    Findings(Vec<Finding>),
    Flash(String),
    Tick,
    Disconnected,
    /// The supervisor this TUI hosts was told to shut down.
    HostStopped,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Effect {
    Call(Call),
    FetchTrace(String),
    Quit,
}

pub struct App {
    pub units: Vec<UnitStatus>,
    pub selected: usize,
    pub mode: Mode,
    pub focus: Pane,
    pub scope: LogScope,
    pub filter: Filter,
    pub filter_text: String,
    pub input: Option<LineInput>,
    pub filter_error: Option<String>,
    pub logs: VecDeque<LogRecord>,
    pub requests: Requests,
    pub findings: Vec<Finding>,
    pub finding_selected: usize,
    pub pending_fix: Option<Finding>,
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
            mode: Mode::Logs,
            focus: Pane::Units,
            scope: LogScope::All,
            filter: Filter::default(),
            filter_text: String::new(),
            input: None,
            filter_error: None,
            logs: VecDeque::new(),
            requests: Requests::default(),
            findings: Vec::new(),
            finding_selected: 0,
            pending_fix: None,
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
            Msg::Scroll { pane, down } => return self.step(pane, down, WHEEL_STEP),
            Msg::Trace(trace) => self.requests.set_trace(trace),
            Msg::Focus(pane) => self.focus = pane,
            Msg::Findings(findings) => {
                self.finding_selected = self.finding_selected.min(findings.len().saturating_sub(1));
                self.findings = findings;
            }
            Msg::State(status) => match self.units.iter_mut().find(|u| u.name == status.name) {
                Some(unit) => *unit = status,
                None => self.units.push(status),
            },
            Msg::Log(record) => {
                let added = self.requests.observe(&record);
                let evicted = if self.logs.len() == LOG_CAPACITY {
                    self.logs.pop_front()
                } else {
                    None
                };
                if self.scroll > 0 && self.matches(&record) {
                    self.scroll += 1;
                }
                self.logs.push_back(record);
                // max_scroll() walks the whole buffer; the match count only shrinks when a
                // matching record fell off the front.
                if self.scroll > 0 && evicted.is_some_and(|old| self.matches(&old)) {
                    self.scroll = self.scroll.min(self.max_scroll());
                }
                if added && self.mode == Mode::Requests && !self.requests.is_pinned() {
                    return self.requests.pin_selection().map(Effect::FetchTrace);
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
        if let Some(finding) = self.pending_fix.take() {
            if key.code == KeyCode::Char('y') {
                return Some(Effect::Call(Call::Fix {
                    finding: finding.id,
                    confirmed: true,
                }));
            }
            self.flash = Some("fix cancelled".to_owned());
            return None;
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let logs_mode = self.mode == Mode::Logs;
        match key.code {
            KeyCode::Char('q') => return Some(Effect::Quit),
            KeyCode::Char('c') if ctrl => return Some(Effect::Quit),
            KeyCode::Char('d') if ctrl => self.page(true, HALF_PAGE),
            KeyCode::Char('u') if ctrl => self.page(false, HALF_PAGE),
            KeyCode::Left | KeyCode::Char('h' | '[') => self.focus = self.side_pane(false),
            KeyCode::Right | KeyCode::Char('l' | ']') => self.focus = self.side_pane(true),
            KeyCode::Down | KeyCode::Char('j') => return self.step(self.focus, true, 1),
            KeyCode::Up | KeyCode::Char('k') => return self.step(self.focus, false, 1),
            KeyCode::PageUp => self.page(false, PAGE),
            KeyCode::PageDown => self.page(true, PAGE),
            KeyCode::Home | KeyCode::Char('g') => self.jump(true),
            KeyCode::End | KeyCode::Char('G') => self.jump(false),
            KeyCode::Char('?') => self.show_help = !self.show_help,
            KeyCode::Char('t') => return self.switch(Mode::Requests),
            KeyCode::Char('d') if !ctrl => return self.switch(Mode::Doctor),
            KeyCode::Esc if !logs_mode => return self.switch(Mode::Logs),
            KeyCode::Char('o') if self.mode == Mode::Requests => self.requests.toggle_order(),
            KeyCode::Char('f') => self.ask_fix(),
            KeyCode::Char('r') if self.mode == Mode::Doctor => {
                return Some(Effect::Call(Call::Doctor { deep: false }));
            }
            KeyCode::Char('R') if self.mode == Mode::Doctor => {
                return Some(Effect::Call(Call::Doctor { deep: true }));
            }
            KeyCode::Enter if self.focus == Pane::Requests => self.focus = Pane::Trace,
            KeyCode::Enter if self.focus == Pane::Units => {
                self.scope = LogScope::Selected;
                self.scroll = 0;
            }
            KeyCode::Char('/') if logs_mode => {
                self.input = Some(LineInput::new(&self.filter_text));
            }
            KeyCode::Tab if logs_mode => {
                self.scope = match self.scope {
                    LogScope::All => LogScope::Selected,
                    LogScope::Selected => LogScope::All,
                };
                self.scroll = 0;
            }
            KeyCode::Char('r') if logs_mode => {
                return self.call(|unit| Call::Restart {
                    unit,
                    cascade: false,
                });
            }
            KeyCode::Char('R') if logs_mode => {
                return self.call(|unit| Call::Restart {
                    unit,
                    cascade: true,
                });
            }
            KeyCode::Char('s') if logs_mode => {
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

    fn step(&mut self, pane: Pane, down: bool, lines: usize) -> Option<Effect> {
        match pane {
            Pane::Units => self.select(down),
            Pane::Logs if down => self.scroll_down(lines),
            Pane::Logs => self.scroll_up(lines),
            Pane::Requests => return self.requests.select(down).map(Effect::FetchTrace),
            Pane::Trace => self.requests.scroll_trace(down, lines),
            Pane::Findings => self.select_finding(down),
        }
        None
    }

    fn side_pane(&self, right: bool) -> Pane {
        match (self.mode, right) {
            (Mode::Logs, false) => Pane::Units,
            (Mode::Logs, true) => Pane::Logs,
            (Mode::Requests, false) => Pane::Requests,
            (Mode::Requests, true) => Pane::Trace,
            (Mode::Doctor, _) => Pane::Findings,
        }
    }

    /// Opens `mode`, or returns to the logs when it is already open.
    fn switch(&mut self, mode: Mode) -> Option<Effect> {
        let target = if self.mode == mode { Mode::Logs } else { mode };
        self.mode = target;
        self.focus = match target {
            Mode::Logs => Pane::Units,
            Mode::Requests => Pane::Requests,
            Mode::Doctor => Pane::Findings,
        };
        if target == Mode::Requests {
            return self.requests.pin_selection().map(Effect::FetchTrace);
        }
        None
    }

    fn page(&mut self, down: bool, lines: usize) {
        match self.mode {
            Mode::Logs if down => self.scroll_down(lines),
            Mode::Logs => self.scroll_up(lines),
            Mode::Requests => self.requests.scroll_trace(down, lines),
            Mode::Doctor => {}
        }
    }

    fn jump(&mut self, top: bool) {
        match self.mode {
            Mode::Logs => self.scroll = if top { self.max_scroll() } else { 0 },
            Mode::Requests => self.requests.jump_trace(top),
            Mode::Doctor => {
                self.finding_selected = if top {
                    0
                } else {
                    self.findings.len().saturating_sub(1)
                }
            }
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

    fn select_finding(&mut self, down: bool) {
        let last = self.findings.len().saturating_sub(1);
        self.finding_selected = if down {
            (self.finding_selected + 1).min(last)
        } else {
            self.finding_selected.saturating_sub(1)
        };
    }

    pub fn unit_findings(&self, unit: &str) -> Vec<&Finding> {
        self.findings
            .iter()
            .filter(|finding| finding.unit.as_deref() == Some(unit))
            .collect()
    }

    fn ask_fix(&mut self) {
        let finding = match self.mode {
            Mode::Doctor => self.findings.get(self.finding_selected).cloned(),
            _ => self.selected_unit().and_then(|unit| {
                self.unit_findings(&unit.name)
                    .into_iter()
                    .find(|finding| finding.fix.is_some())
                    .cloned()
            }),
        };
        match finding {
            Some(finding) if finding.fix.is_some() => self.pending_fix = Some(finding),
            Some(_) => self.flash = Some("this finding has no automatic fix".to_owned()),
            None => self.flash = Some("nothing to fix here".to_owned()),
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
        doctor::{Fix, Severity},
        logs::{
            parse_line,
            trace::{Trace, tests::finished_request},
        },
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

    #[test]
    fn evicting_a_non_matching_record_leaves_a_scrolled_view_alone() {
        let mut app = app_with_two_units();
        app.update(Msg::Log(record("backend", "oldest")));
        for i in 1..LOG_CAPACITY {
            app.update(Msg::Log(record("db", &i.to_string())));
        }
        app.update(key(KeyCode::Char('/')));
        type_text(&mut app, "unit:db");
        app.update(key(KeyCode::Enter));
        app.update(key(KeyCode::Char('l')));
        app.update(key(KeyCode::Char('k')));
        let scroll = app.scroll;
        let before: Vec<String> = app
            .visible_logs(5)
            .iter()
            .map(|r| r.message.clone())
            .collect();
        assert!(scroll > 0);

        app.update(Msg::Log(record("db", "new")));

        assert_eq!(app.logs.len(), LOG_CAPACITY);
        assert_eq!(app.scroll, scroll + 1);
        let after: Vec<String> = app
            .visible_logs(5)
            .iter()
            .map(|r| r.message.clone())
            .collect();
        assert_eq!(before, after);
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

    #[test]
    fn t_opens_the_requests_and_fetches_the_newest_trace() {
        let mut app = app_with_two_units();
        app.update(Msg::Log(finished_request("old", "/a", 200, 3)));
        app.update(Msg::Log(finished_request("new", "/b", 500, 9)));

        assert_eq!(
            app.update(key(KeyCode::Char('t'))),
            Some(Effect::FetchTrace("new".into()))
        );
        assert_eq!((app.mode, app.focus), (Mode::Requests, Pane::Requests));
        assert_eq!(
            app.update(key(KeyCode::Char('j'))),
            Some(Effect::FetchTrace("old".into()))
        );
        assert_eq!(
            app.update(key(KeyCode::Char('r'))),
            None,
            "no unit actions here"
        );

        app.update(Msg::Trace(Trace::build(
            "old",
            &[finished_request("old", "/a", 200, 3)],
        )));
        assert!(app.requests.trace.is_some());

        app.update(key(KeyCode::Enter));
        assert_eq!(app.focus, Pane::Trace);
        app.update(key(KeyCode::Char('h')));
        assert_eq!(app.focus, Pane::Requests);

        app.update(key(KeyCode::Esc));
        assert_eq!((app.mode, app.focus), (Mode::Logs, Pane::Units));
    }

    #[test]
    fn the_first_request_arriving_in_an_empty_view_is_fetched_once() {
        let mut app = app_with_two_units();
        assert_eq!(app.update(key(KeyCode::Char('t'))), None);
        assert_eq!(app.mode, Mode::Requests);

        assert_eq!(
            app.update(Msg::Log(finished_request("first", "/a", 200, 1))),
            Some(Effect::FetchTrace("first".into()))
        );
        assert_eq!(app.logs.len(), 1);
        assert_eq!(
            app.update(Msg::Log(finished_request("second", "/b", 200, 1))),
            None
        );
        assert_eq!(app.logs.len(), 2);
    }

    #[test]
    fn the_wheel_over_the_request_list_selects_and_fetches() {
        let mut app = app_with_two_units();
        app.update(Msg::Log(finished_request("a", "/a", 200, 3)));
        app.update(Msg::Log(finished_request("b", "/b", 200, 3)));
        app.update(key(KeyCode::Char('t')));

        assert_eq!(
            app.update(Msg::Scroll {
                pane: Pane::Requests,
                down: true
            }),
            Some(Effect::FetchTrace("a".into()))
        );
    }

    fn findings() -> Vec<Finding> {
        vec![
            Finding::new(
                "dev-db-migrations",
                Severity::Error,
                "dev database is 2 migration(s) behind",
            )
            .unit("backend")
            .fix(Fix::just("migrate-up")),
            Finding::new("ports", Severity::Error, "port 5432 is taken")
                .subject("5432")
                .unit("db"),
            Finding::new("testcontainers", Severity::Hint, "3 leftovers")
                .fix(Fix::just("test-clean").destructive()),
        ]
    }

    #[test]
    fn d_opens_the_doctor_and_f_fixes_after_confirmation() {
        let mut app = app_with_two_units();
        app.update(Msg::Findings(findings()));

        app.update(key(KeyCode::Char('d')));
        assert_eq!((app.mode, app.focus), (Mode::Doctor, Pane::Findings));

        app.update(key(KeyCode::Char('f')));
        assert_eq!(
            app.pending_fix.as_ref().map(|f| f.id.as_str()),
            Some("dev-db-migrations")
        );
        assert_eq!(
            app.update(key(KeyCode::Char('y'))),
            Some(Effect::Call(Call::Fix {
                finding: "dev-db-migrations".into(),
                confirmed: true
            }))
        );
        assert!(app.pending_fix.is_none());

        app.update(key(KeyCode::Char('j')));
        app.update(key(KeyCode::Char('f')));
        assert_eq!(
            app.flash.as_deref(),
            Some("this finding has no automatic fix")
        );

        app.update(key(KeyCode::Char('j')));
        app.update(key(KeyCode::Char('f')));
        assert_eq!(app.update(key(KeyCode::Char('n'))), None);
        assert_eq!(app.flash.as_deref(), Some("fix cancelled"));

        assert_eq!(
            app.update(key(KeyCode::Char('R'))),
            Some(Effect::Call(Call::Doctor { deep: true }))
        );
        app.update(key(KeyCode::Esc));
        assert_eq!(app.mode, Mode::Logs);
    }

    #[test]
    fn f_in_the_logs_fixes_the_selected_units_finding() {
        let mut app = app_with_two_units();
        app.update(Msg::State(status(
            "backend",
            UnitState::Failed { reason: "x".into() },
        )));
        app.update(Msg::Findings(findings()));

        app.update(key(KeyCode::Char('f')));
        assert_eq!(
            app.flash.as_deref(),
            Some("nothing to fix here"),
            "db has no fixable finding"
        );

        app.update(key(KeyCode::Char('j')));
        app.update(key(KeyCode::Char('f')));
        assert_eq!(
            app.pending_fix.as_ref().map(|f| f.id.as_str()),
            Some("dev-db-migrations")
        );
        assert_eq!(app.unit_findings("db").len(), 1);
    }

    #[test]
    fn the_selection_stays_in_range_when_findings_shrink() {
        let mut app = app_with_two_units();
        app.update(Msg::Findings(findings()));
        app.update(key(KeyCode::Char('d')));
        app.update(key(KeyCode::Char('G')));
        app.update(key(KeyCode::Char('j')));
        app.update(key(KeyCode::Char('j')));
        app.update(Msg::Findings(findings()[..1].to_vec()));
        assert_eq!(app.finding_selected, 0);
    }
}
