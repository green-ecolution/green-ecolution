use std::collections::VecDeque;

use chrono::Utc;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{
    input::{InputOutcome, LineInput},
    palette::{self, Action, Palette, PaletteOutcome, ParamPrompt, PromptOutcome, Sources},
    requests::Requests,
};
use crate::{
    doctor::{Finding, Severity},
    ipc::Call,
    logs::{Filter, LogRecord, trace::Trace},
    supervisor::{UnitState, UnitStatus},
    tasks::Recipe,
    units::Origin,
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
    Units(Vec<UnitStatus>),
    Profile(String),
    Profiles(Vec<String>),
    Log(LogRecord),
    Trace(Trace),
    Findings(Vec<Finding>),
    Recipes(Vec<Recipe>),
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
    FetchRecipes,
    Run {
        recipe: String,
        args: Vec<String>,
        confirmed: bool,
    },
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingRun {
    pub recipe: String,
    pub args: Vec<String>,
    pub command: String,
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
    pub profile: Option<String>,
    pub profiles: Vec<String>,
    pub palette: Option<Palette>,
    pub prompt: Option<ParamPrompt>,
    pub pending_run: Option<PendingRun>,
    pub recipes: Vec<Recipe>,
    units_from_snapshot: bool,
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
            profile: None,
            profiles: Vec::new(),
            palette: None,
            prompt: None,
            pending_run: None,
            recipes: Vec::new(),
            units_from_snapshot: false,
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
            Msg::Scroll { .. } | Msg::Focus(_) if self.palette.is_some() => {}
            Msg::Scroll { pane, down } => return self.step(pane, down, WHEEL_STEP),
            Msg::Trace(trace) => self.requests.set_trace(trace),
            Msg::Focus(pane) => self.focus = pane,
            Msg::Findings(findings) => {
                self.finding_selected = self.finding_selected.min(findings.len().saturating_sub(1));
                self.findings = findings;
            }
            Msg::State(status)
                if status.origin == Origin::Task
                    && matches!(status.state, UnitState::Done | UnitState::Stopped) =>
            {
                self.remove_unit(&status.name);
                if status.state == UnitState::Done {
                    self.flash = Some(format!("{} done", status.name));
                }
            }
            Msg::State(status) => match self.units.iter_mut().find(|u| u.name == status.name) {
                Some(unit) => *unit = status,
                // A snapshot is the complete list of profile units, but tasks come and go.
                None if !self.units_from_snapshot || status.origin == Origin::Task => {
                    self.units.push(status);
                }
                None => {}
            },
            Msg::Recipes(recipes) if recipes == self.recipes => {}
            Msg::Recipes(recipes) => {
                self.recipes = recipes;
                let entries = self.palette_entries();
                if let Some(palette) = &mut self.palette {
                    palette.set_entries(entries);
                }
            }
            Msg::Units(units) => {
                self.units_from_snapshot = true;
                let previous = self.selected_unit().map(|unit| unit.name.as_str());
                self.selected = previous
                    .and_then(|name| units.iter().position(|unit| unit.name == name))
                    .unwrap_or_else(|| self.selected.min(units.len().saturating_sub(1)));
                self.units = units;
            }
            Msg::Profile(profile) => {
                if self.profile.as_ref().is_some_and(|known| *known != profile) {
                    self.flash = Some(format!("switched to {profile}"));
                }
                self.profile = Some(profile);
            }
            Msg::Profiles(profiles) => self.profiles = profiles,
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
        if let Some(prompt) = &mut self.prompt {
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                return Some(Effect::Quit);
            }
            return match prompt.handle(key) {
                PromptOutcome::Editing => None,
                PromptOutcome::Cancelled => {
                    self.prompt = None;
                    None
                }
                PromptOutcome::Done(args) => {
                    let recipe = prompt.recipe().clone();
                    self.prompt = None;
                    self.run_recipe(recipe, args)
                }
            };
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
        if let Some(run) = self.pending_run.take() {
            if key.code == KeyCode::Char('y') {
                self.flash = Some(format!("running {}…", run.recipe));
                return Some(Effect::Run {
                    recipe: run.recipe,
                    args: run.args,
                    confirmed: true,
                });
            }
            self.flash = Some("run cancelled".to_owned());
            return None;
        }
        if let Some(palette) = &mut self.palette {
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                return Some(Effect::Quit);
            }
            return match palette.handle(key) {
                PaletteOutcome::Open => None,
                PaletteOutcome::Cancelled => {
                    self.palette = None;
                    None
                }
                PaletteOutcome::Chosen(action) => {
                    self.palette = None;
                    self.act(action)
                }
            };
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
            KeyCode::Char('k') if ctrl => return self.open_palette(""),
            KeyCode::Char(':') => return self.open_palette(""),
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
            KeyCode::Char('p') if !ctrl => return self.open_palette("profile "),
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

    /// The finding a unit's status line shows and `f` acts on: a fixable error first,
    /// then any fixable one, then the worst.
    pub fn unit_finding(&self, unit: &str) -> Option<&Finding> {
        let all = self.unit_findings(unit);
        all.iter()
            .copied()
            .find(|f| f.fix.is_some() && f.severity == Severity::Error)
            .or_else(|| all.iter().copied().find(|f| f.fix.is_some()))
            .or_else(|| {
                all.iter()
                    .copied()
                    .reduce(|best, f| if f.severity > best.severity { f } else { best })
            })
    }

    fn palette_entries(&self) -> Vec<palette::Entry> {
        palette::entries(&Sources {
            units: &self.units,
            profiles: &self.profiles,
            active: self.profile.as_deref(),
            recipes: &self.recipes,
            findings: &self.findings,
            filtered: !self.filter_text.is_empty(),
        })
    }

    /// Fetches the recipes again, so an edited justfile shows up without a restart.
    fn open_palette(&mut self, query: &str) -> Option<Effect> {
        self.palette = Some(Palette::new(self.palette_entries(), query));
        Some(Effect::FetchRecipes)
    }

    fn act(&mut self, action: Action) -> Option<Effect> {
        match action {
            Action::Start(unit) => Some(Effect::Call(Call::Start { unit })),
            Action::Stop(unit) => Some(Effect::Call(Call::Stop { unit })),
            Action::Restart { unit, cascade } => {
                Some(Effect::Call(Call::Restart { unit, cascade }))
            }
            Action::Profile(profile) if self.profile.as_ref() == Some(&profile) => {
                self.flash = Some(format!("already on {profile}"));
                None
            }
            Action::Profile(profile) => {
                self.flash = Some(format!("switching to {profile}…"));
                Some(Effect::Call(Call::SwitchProfile { profile }))
            }
            Action::Fix(finding) => {
                self.pending_fix = Some(finding);
                None
            }
            Action::Filter => {
                self.mode = Mode::Logs;
                self.focus = Pane::Units;
                self.input = Some(LineInput::new(&self.filter_text));
                None
            }
            Action::ClearFilter => {
                self.apply_filter("");
                None
            }
            Action::Run(recipe) => match ParamPrompt::new(recipe.clone()) {
                Some(prompt) => {
                    self.prompt = Some(prompt);
                    None
                }
                None => self.run_recipe(recipe, Vec::new()),
            },
        }
    }

    fn run_recipe(&mut self, recipe: Recipe, args: Vec<String>) -> Option<Effect> {
        if recipe.destructive {
            self.pending_run = Some(PendingRun {
                command: recipe.command("just", &args),
                recipe: recipe.name,
                args,
            });
            return None;
        }
        self.flash = Some(format!("running {}…", recipe.name));
        Some(Effect::Run {
            recipe: recipe.name,
            args,
            confirmed: false,
        })
    }

    fn remove_unit(&mut self, name: &str) {
        let selected = self.selected_unit().map(|unit| unit.name.clone());
        self.units.retain(|unit| unit.name != name);
        self.selected = selected
            .and_then(|selected| self.units.iter().position(|unit| unit.name == selected))
            .unwrap_or_else(|| self.selected.min(self.units.len().saturating_sub(1)));
    }

    fn ask_fix(&mut self) {
        let finding = match self.mode {
            Mode::Doctor => self.findings.get(self.finding_selected).cloned(),
            _ => self
                .selected_unit()
                .and_then(|unit| self.unit_finding(&unit.name).cloned()),
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
        doctor::Fix,
        logs::{
            parse_line,
            trace::{Trace, tests::finished_request},
        },
        tasks::{Param, ParamKind, Recipe},
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

    fn task(name: &str, state: UnitState) -> UnitStatus {
        UnitStatus {
            origin: Origin::Task,
            ..status(name, state)
        }
    }

    fn recipe(name: &str) -> Recipe {
        Recipe {
            name: name.into(),
            doc: None,
            group: None,
            private: false,
            destructive: false,
            params: Vec::new(),
        }
    }

    fn type_text(app: &mut App, text: &str) {
        for c in text.chars() {
            app.update(key(KeyCode::Char(c)));
        }
    }

    #[test]
    fn p_opens_the_palette_on_the_profiles() {
        let mut app = app_with_two_units();
        app.update(Msg::Profiles(vec!["full".into(), "minimal".into()]));
        app.update(Msg::Profile("minimal".into()));

        assert_eq!(
            app.update(key(KeyCode::Char('p'))),
            Some(Effect::FetchRecipes)
        );
        assert_eq!(
            app.update(key(KeyCode::Char('q'))),
            None,
            "keys go to the palette"
        );
        assert!(app.palette.is_some());
        app.update(key(KeyCode::Backspace));
        assert_eq!(
            app.update(key(KeyCode::Enter)),
            Some(Effect::Call(Call::SwitchProfile {
                profile: "full".into()
            }))
        );
        assert!(app.palette.is_none());
        assert_eq!(app.flash.as_deref(), Some("switching to full…"));
    }

    #[test]
    fn choosing_the_active_profile_sends_nothing() {
        let mut app = app_with_two_units();
        app.update(Msg::Profiles(vec!["full".into(), "minimal".into()]));
        app.update(Msg::Profile("minimal".into()));
        app.update(key(KeyCode::Char('p')));
        app.update(key(KeyCode::Down));
        assert_eq!(app.update(key(KeyCode::Enter)), None);
        assert_eq!(app.flash.as_deref(), Some("already on minimal"));
    }

    #[test]
    fn colon_and_ctrl_k_open_the_palette() {
        let mut app = app_with_two_units();
        assert_eq!(
            app.update(key(KeyCode::Char(':'))),
            Some(Effect::FetchRecipes)
        );
        app.update(key(KeyCode::Esc));
        assert!(app.palette.is_none());
        assert_eq!(
            app.update(ctrl(KeyCode::Char('k'))),
            Some(Effect::FetchRecipes)
        );
        assert!(app.palette.is_some());
        assert_eq!(app.selected, 0, "ctrl-k does not move the selection");
    }

    #[test]
    fn the_palette_blocks_the_mouse_but_not_ctrl_c() {
        let mut app = app_with_two_units();
        app.update(key(KeyCode::Char(':')));
        app.update(Msg::Scroll {
            pane: Pane::Units,
            down: true,
        });
        app.update(Msg::Focus(Pane::Logs));
        assert_eq!(app.selected, 0);
        assert_eq!(app.focus, Pane::Units);
        assert_eq!(app.update(ctrl(KeyCode::Char('c'))), Some(Effect::Quit));
    }

    #[test]
    fn the_palette_restarts_units() {
        let mut app = app_with_two_units();
        app.update(key(KeyCode::Char(':')));
        type_text(&mut app, "restart db");
        assert_eq!(
            app.update(key(KeyCode::Enter)),
            Some(Effect::Call(Call::Restart {
                unit: "db".into(),
                cascade: false
            }))
        );
    }

    #[test]
    fn a_recipe_without_parameters_runs_right_away() {
        let mut app = app_with_two_units();
        app.update(Msg::Recipes(vec![recipe("migrate-up")]));
        app.update(key(KeyCode::Char(':')));
        type_text(&mut app, "run migrate-up");
        assert_eq!(
            app.update(key(KeyCode::Enter)),
            Some(Effect::Run {
                recipe: "migrate-up".into(),
                args: Vec::new(),
                confirmed: false
            })
        );
    }

    #[test]
    fn a_recipe_with_parameters_asks_for_them_inline() {
        let mut app = app_with_two_units();
        let mut new = recipe("migrate-new");
        new.params = vec![Param {
            name: "name".into(),
            default: None,
            optional: false,
            kind: ParamKind::Singular,
        }];
        app.update(Msg::Recipes(vec![new]));
        app.update(key(KeyCode::Char(':')));
        type_text(&mut app, "migrate-new");
        assert_eq!(app.update(key(KeyCode::Enter)), None);
        assert!(app.prompt.is_some());

        type_text(&mut app, "add_foo");
        assert_eq!(
            app.update(key(KeyCode::Enter)),
            Some(Effect::Run {
                recipe: "migrate-new".into(),
                args: vec!["add_foo".into()],
                confirmed: false
            })
        );
        assert!(app.prompt.is_none());
    }

    #[test]
    fn ctrl_c_quits_from_the_parameter_prompt() {
        let mut app = app_with_two_units();
        let mut new = recipe("migrate-new");
        new.params = vec![Param {
            name: "name".into(),
            default: None,
            optional: false,
            kind: ParamKind::Singular,
        }];
        app.update(Msg::Recipes(vec![new]));
        app.update(key(KeyCode::Char(':')));
        type_text(&mut app, "migrate-new");
        app.update(key(KeyCode::Enter));
        assert!(app.prompt.is_some());

        assert_eq!(app.update(ctrl(KeyCode::Char('c'))), Some(Effect::Quit));
    }

    #[test]
    fn destructive_recipes_ask_for_confirmation() {
        let mut app = app_with_two_units();
        let mut reset = recipe("migrate-reset");
        reset.destructive = true;
        app.update(Msg::Recipes(vec![reset]));

        app.update(key(KeyCode::Char(':')));
        type_text(&mut app, "migrate-reset");
        assert_eq!(app.update(key(KeyCode::Enter)), None);
        assert_eq!(
            app.pending_run.as_ref().unwrap().command,
            "just migrate-reset"
        );
        assert_eq!(
            app.update(key(KeyCode::Char('y'))),
            Some(Effect::Run {
                recipe: "migrate-reset".into(),
                args: Vec::new(),
                confirmed: true
            })
        );

        app.update(key(KeyCode::Char(':')));
        type_text(&mut app, "migrate-reset");
        app.update(key(KeyCode::Enter));
        assert_eq!(app.update(key(KeyCode::Char('n'))), None);
        assert_eq!(app.flash.as_deref(), Some("run cancelled"));
    }

    #[test]
    fn finished_and_stopped_tasks_leave_the_list_failed_ones_stay() {
        let mut app = app_with_two_units();
        app.update(Msg::Units(vec![
            status("db", UnitState::Ready),
            status("backend", UnitState::Ready),
        ]));
        app.update(Msg::State(task("migrate-up", UnitState::Starting)));
        assert_eq!(app.units.len(), 3, "tasks are listed after a snapshot too");

        app.update(Msg::State(task("migrate-up", UnitState::Done)));
        assert_eq!(app.units.len(), 2);
        assert_eq!(app.flash.as_deref(), Some("migrate-up done"));

        app.update(Msg::State(task(
            "lint",
            UnitState::Failed {
                reason: "exit 1".into(),
            },
        )));
        assert_eq!(app.units.len(), 3);
        app.update(Msg::State(task("lint", UnitState::Stopped)));
        assert_eq!(app.units.len(), 2);
    }

    #[test]
    fn removing_a_task_keeps_the_selected_unit() {
        let mut app = app_with_two_units();
        app.update(Msg::State(task("lint", UnitState::Starting)));
        app.update(Msg::State(task("fmt", UnitState::Starting)));
        for _ in 0..3 {
            app.update(key(KeyCode::Char('j')));
        }
        assert_eq!(app.selected_unit().unwrap().name, "fmt");
        app.update(Msg::State(task("lint", UnitState::Done)));
        assert_eq!(app.selected_unit().unwrap().name, "fmt");
    }

    #[test]
    fn the_recipe_reply_keeps_the_palette_selection() {
        let mut app = app_with_two_units();
        app.update(Msg::Recipes(vec![recipe("alpha"), recipe("beta")]));
        app.update(key(KeyCode::Char(':')));
        type_text(&mut app, "run ");
        app.update(key(KeyCode::Down));
        let second = app.palette.as_ref().unwrap().matches()[1].label.clone();
        app.update(Msg::Recipes(vec![recipe("alpha"), recipe("beta")]));
        app.update(Msg::Recipes(vec![
            recipe("alpha"),
            recipe("beta"),
            recipe("gamma"),
        ]));
        let palette = app.palette.as_ref().unwrap();
        assert_eq!(palette.matches()[palette.selected()].label, second);
        let effect = app.update(key(KeyCode::Enter));
        assert_eq!(
            effect,
            Some(Effect::Run {
                recipe: second.trim_start_matches("run ").into(),
                args: Vec::new(),
                confirmed: false
            })
        );
    }

    #[test]
    fn the_palette_offers_fixes_and_filters() {
        let mut app = app_with_two_units();
        app.update(Msg::Findings(vec![
            Finding::new("dist", Severity::Error, "ui dist is stale").fix(Fix::just("build-ui")),
        ]));
        app.update(key(KeyCode::Char(':')));
        type_text(&mut app, "fix ui");
        assert_eq!(app.update(key(KeyCode::Enter)), None);
        assert!(app.pending_fix.is_some(), "fixes keep their y/N question");
        app.update(key(KeyCode::Char('n')));

        app.update(key(KeyCode::Char(':')));
        type_text(&mut app, "filter logs");
        app.update(key(KeyCode::Enter));
        assert!(app.input.is_some());
    }

    #[test]
    fn a_new_profile_replaces_the_switching_flash() {
        let mut app = app_with_two_units();
        app.update(Msg::Profile("minimal".into()));
        assert_eq!(app.flash, None, "the first profile is no switch");

        app.update(Msg::Flash("switching to full…".into()));
        app.update(Msg::Profile("full".into()));
        assert_eq!(app.flash.as_deref(), Some("switched to full"));

        app.update(Msg::Flash("something else".into()));
        app.update(Msg::Profile("full".into()));
        assert_eq!(app.flash.as_deref(), Some("something else"));
    }

    #[test]
    fn a_unit_snapshot_keeps_the_selected_unit() {
        let mut app = app_with_two_units();
        app.update(key(KeyCode::Char('j')));
        assert_eq!(app.selected_unit().unwrap().name, "backend");

        app.update(Msg::Units(vec![
            status("db", UnitState::Ready),
            status("build-release", UnitState::Done),
            status("backend", UnitState::Ready),
        ]));
        assert_eq!(app.selected_unit().unwrap().name, "backend");
    }

    #[test]
    fn a_unit_snapshot_replaces_the_list() {
        let mut app = app_with_two_units();
        app.update(key(KeyCode::Char('j')));
        app.update(Msg::Units(vec![status("db", UnitState::Ready)]));
        assert_eq!(app.units.len(), 1);
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn a_late_state_does_not_readd_a_unit_the_snapshot_dropped() {
        let mut app = app_with_two_units();
        app.update(Msg::Units(vec![status("db", UnitState::Ready)]));
        app.update(Msg::State(status("backend", UnitState::Stopped)));
        assert_eq!(app.units.len(), 1);
        app.update(Msg::State(status("db", UnitState::Stopped)));
        assert_eq!(app.units[0].state, UnitState::Stopped);
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
            Some("this finding has no automatic fix"),
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

    #[test]
    fn the_status_finding_and_f_agree_on_the_fixable_error() {
        let mut app = app_with_two_units();
        app.update(Msg::Findings(vec![
            Finding::new("hint", Severity::Hint, "just a hint").unit("db"),
            Finding::new("broken", Severity::Error, "broken thing")
                .unit("db")
                .fix(Fix::just("repair")),
        ]));
        assert_eq!(
            app.unit_finding("db").map(|f| f.id.as_str()),
            Some("broken")
        );
        app.update(key(KeyCode::Char('f')));
        assert_eq!(
            app.pending_fix.as_ref().map(|f| f.id.as_str()),
            Some("broken")
        );
    }
}
