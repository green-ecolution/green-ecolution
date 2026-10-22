use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::input::{InputOutcome, LineInput};
use crate::{
    doctor::Finding,
    fuzzy,
    supervisor::{UnitState, UnitStatus},
    tasks::{Param, ParamKind, Recipe},
    units::Origin,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Start(String),
    Stop(String),
    Restart { unit: String, cascade: bool },
    Profile(String),
    Run(Recipe),
    Fix(Finding),
    Filter,
    ClearFilter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub label: String,
    pub hint: String,
    pub action: Action,
}

impl Entry {
    fn new(label: String, hint: String, action: Action) -> Self {
        Self {
            label,
            hint,
            action,
        }
    }
}

pub struct Sources<'a> {
    pub units: &'a [UnitStatus],
    pub profiles: &'a [String],
    pub active: Option<&'a str>,
    pub recipes: &'a [Recipe],
    pub findings: &'a [Finding],
    pub filtered: bool,
}

pub fn entries(sources: &Sources<'_>) -> Vec<Entry> {
    let mut entries = Vec::new();
    for unit in sources.units {
        let name = &unit.name;
        let state = unit.state.to_string();
        let stopped = unit.state == UnitState::Stopped;
        if stopped || matches!(unit.state, UnitState::Failed { .. }) {
            entries.push(Entry::new(
                format!("start {name}"),
                state.clone(),
                Action::Start(name.clone()),
            ));
        }
        if !stopped {
            entries.push(Entry::new(
                format!("stop {name}"),
                state.clone(),
                Action::Stop(name.clone()),
            ));
        }
        entries.push(Entry::new(
            format!("restart {name}"),
            state.clone(),
            Action::Restart {
                unit: name.clone(),
                cascade: false,
            },
        ));
        if unit.origin != Origin::Task {
            entries.push(Entry::new(
                format!("restart {name} with dependents"),
                state,
                Action::Restart {
                    unit: name.clone(),
                    cascade: true,
                },
            ));
        }
    }
    for profile in sources.profiles {
        let hint = if sources.active == Some(profile.as_str()) {
            "active"
        } else {
            ""
        };
        entries.push(Entry::new(
            format!("profile {profile}"),
            hint.to_owned(),
            Action::Profile(profile.clone()),
        ));
    }
    for recipe in sources.recipes.iter().filter(|recipe| !recipe.private) {
        let hint = [
            recipe.group.as_deref(),
            recipe.doc.as_deref(),
            recipe.destructive.then_some("destructive"),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" · ");
        entries.push(Entry::new(
            format!("run {}", recipe.signature()),
            hint,
            Action::Run(recipe.clone()),
        ));
    }
    for finding in sources.findings {
        if let Some(fix) = &finding.fix {
            entries.push(Entry::new(
                format!("fix: {}", finding.title),
                fix.command.clone(),
                Action::Fix(finding.clone()),
            ));
        }
    }
    entries.push(Entry::new(
        "filter logs".to_owned(),
        r#"level>=warn unit:backend "text""#.to_owned(),
        Action::Filter,
    ));
    if sources.filtered {
        entries.push(Entry::new(
            "clear filter".to_owned(),
            String::new(),
            Action::ClearFilter,
        ));
    }
    entries
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaletteOutcome {
    Open,
    Chosen(Action),
    Cancelled,
}

pub struct Palette {
    input: LineInput,
    entries: Vec<Entry>,
    matches: Vec<usize>,
    selected: usize,
}

impl Palette {
    pub fn new(entries: Vec<Entry>, query: &str) -> Self {
        let mut palette = Self {
            input: LineInput::new(query),
            entries,
            matches: Vec::new(),
            selected: 0,
        };
        palette.refresh();
        palette
    }

    pub fn input(&self) -> &LineInput {
        &self.input
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn matches(&self) -> Vec<&Entry> {
        self.matches
            .iter()
            .map(|&index| &self.entries[index])
            .collect()
    }

    pub fn set_entries(&mut self, entries: Vec<Entry>) {
        self.entries = entries;
        self.refresh();
    }

    pub fn handle(&mut self, key: KeyEvent) -> PaletteOutcome {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Esc => return PaletteOutcome::Cancelled,
            KeyCode::Enter => {
                return self
                    .matches
                    .get(self.selected)
                    .map_or(PaletteOutcome::Open, |&index| {
                        PaletteOutcome::Chosen(self.entries[index].action.clone())
                    });
            }
            KeyCode::Down => self.step(true),
            KeyCode::Char('n') if ctrl => self.step(true),
            KeyCode::Up => self.step(false),
            KeyCode::Char('p') if ctrl => self.step(false),
            _ => {
                let before = self.input.text().to_owned();
                self.input.handle(key);
                if self.input.text() != before {
                    self.refresh();
                }
            }
        }
        PaletteOutcome::Open
    }

    fn step(&mut self, down: bool) {
        let last = self.matches.len().saturating_sub(1);
        self.selected = if down {
            (self.selected + 1).min(last)
        } else {
            self.selected.saturating_sub(1)
        };
    }

    fn refresh(&mut self) {
        self.matches = fuzzy::rank(self.input.text(), &self.entries, |entry| {
            entry.label.as_str()
        });
        self.selected = 0;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptOutcome {
    Editing,
    Done(Vec<String>),
    Cancelled,
}

/// Asks a recipe's parameters one after the other in the status line.
pub struct ParamPrompt {
    recipe: Recipe,
    index: usize,
    args: Vec<String>,
    input: LineInput,
    error: Option<String>,
}

impl ParamPrompt {
    pub fn new(recipe: Recipe) -> Option<Self> {
        let first = recipe.params.first()?;
        let input = LineInput::new(first.default.as_deref().unwrap_or(""));
        Some(Self {
            recipe,
            index: 0,
            args: Vec::new(),
            input,
            error: None,
        })
    }

    pub fn recipe(&self) -> &Recipe {
        &self.recipe
    }

    pub fn param(&self) -> &Param {
        &self.recipe.params[self.index]
    }

    pub fn input(&self) -> &LineInput {
        &self.input
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn handle(&mut self, key: KeyEvent) -> PromptOutcome {
        match self.input.handle(key) {
            InputOutcome::Editing => PromptOutcome::Editing,
            InputOutcome::Cancel => PromptOutcome::Cancelled,
            InputOutcome::Submit(text) => self.submit(text.trim()),
        }
    }

    fn submit(&mut self, text: &str) -> PromptOutcome {
        let param = self.param().clone();
        if text.is_empty() {
            if param.optional {
                // Positional: once one is left out, just fills in the rest from their defaults.
                return PromptOutcome::Done(std::mem::take(&mut self.args));
            }
            self.error = Some(format!("{} is required", param.name));
            return PromptOutcome::Editing;
        }
        match param.kind {
            ParamKind::Singular => self.args.push(text.to_owned()),
            ParamKind::Star | ParamKind::Plus => {
                self.args.extend(text.split_whitespace().map(str::to_owned));
            }
        }
        self.index += 1;
        match self.recipe.params.get(self.index) {
            Some(next) => {
                self.input = LineInput::new(next.default.as_deref().unwrap_or(""));
                self.error = None;
                PromptOutcome::Editing
            }
            None => PromptOutcome::Done(std::mem::take(&mut self.args)),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::*;
    use crate::{
        doctor::{Fix, Severity},
        units::UnitKind,
    };

    fn unit(name: &str, state: UnitState, origin: Origin) -> UnitStatus {
        UnitStatus {
            name: name.into(),
            kind: UnitKind::Service,
            origin,
            state,
            since: Utc::now(),
        }
    }

    fn recipe(name: &str, params: Vec<Param>) -> Recipe {
        Recipe {
            name: name.into(),
            doc: Some(format!("about {name}")),
            group: Some("db".into()),
            private: false,
            destructive: false,
            params,
        }
    }

    fn param(name: &str, default: Option<&str>, optional: bool, kind: ParamKind) -> Param {
        Param {
            name: name.into(),
            default: default.map(str::to_owned),
            optional,
            kind,
        }
    }

    fn sample() -> Vec<Entry> {
        let units = [
            unit("db", UnitState::Ready, Origin::Compose),
            unit("backend", UnitState::Stopped, Origin::Local),
            unit(
                "lint",
                UnitState::Failed { reason: "x".into() },
                Origin::Task,
            ),
        ];
        let mut reset = recipe("migrate-reset", Vec::new());
        reset.destructive = true;
        let mut private = recipe("_compose", Vec::new());
        private.private = true;
        let recipes = [
            recipe("migrate-up", Vec::new()),
            recipe(
                "migrate-new",
                vec![param("name", None, false, ParamKind::Singular)],
            ),
            reset,
            private,
        ];
        let findings = [
            Finding::new("dist", Severity::Error, "ui dist is stale").fix(Fix::just("build-ui"))
        ];
        entries(&Sources {
            units: &units,
            profiles: &["full".into(), "minimal".into()],
            active: Some("minimal"),
            recipes: &recipes,
            findings: &findings,
            filtered: true,
        })
    }

    fn labels(entries: &[&Entry]) -> Vec<String> {
        entries.iter().map(|entry| entry.label.clone()).collect()
    }

    fn press(palette: &mut Palette, code: KeyCode) -> PaletteOutcome {
        palette.handle(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn typed(palette: &mut Palette, text: &str) {
        for c in text.chars() {
            press(palette, KeyCode::Char(c));
        }
    }

    #[test]
    fn lists_every_kind_of_action() {
        let entries = sample();
        let all: Vec<&Entry> = entries.iter().collect();
        assert_eq!(
            labels(&all),
            [
                "stop db",
                "restart db",
                "restart db with dependents",
                "start backend",
                "restart backend",
                "restart backend with dependents",
                "start lint",
                "stop lint",
                "restart lint",
                "profile full",
                "profile minimal",
                "run migrate-up",
                "run migrate-new <name>",
                "run migrate-reset",
                "fix: ui dist is stale",
                "filter logs",
                "clear filter",
            ]
        );
        let reset = entries
            .iter()
            .find(|e| e.label == "run migrate-reset")
            .unwrap();
        assert_eq!(reset.hint, "db · about migrate-reset · destructive");
        let active = entries
            .iter()
            .find(|e| e.label == "profile minimal")
            .unwrap();
        assert_eq!(active.hint, "active");
        let fix = entries.iter().find(|e| e.label.starts_with("fix")).unwrap();
        assert_eq!(fix.hint, "just build-ui");
    }

    #[test]
    fn typing_narrows_and_ranks_the_entries() {
        let mut palette = Palette::new(sample(), "");
        typed(&mut palette, "mig up");
        assert_eq!(labels(&palette.matches())[0], "run migrate-up");
        assert_eq!(palette.selected(), 0);
    }

    #[test]
    fn a_profile_query_shows_only_profiles() {
        let palette = Palette::new(sample(), "profile ");
        assert_eq!(
            labels(&palette.matches()),
            ["profile full", "profile minimal"]
        );
        assert_eq!(palette.input().text(), "profile ");
    }

    #[test]
    fn enter_chooses_the_selected_entry_and_escape_cancels() {
        let mut palette = Palette::new(sample(), "profile ");
        press(&mut palette, KeyCode::Down);
        assert_eq!(
            press(&mut palette, KeyCode::Enter),
            PaletteOutcome::Chosen(Action::Profile("minimal".into()))
        );
        palette.handle(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL));
        assert_eq!(palette.selected(), 0);
        palette.handle(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL));
        assert_eq!(palette.selected(), 1);
        assert_eq!(press(&mut palette, KeyCode::Esc), PaletteOutcome::Cancelled);
    }

    #[test]
    fn enter_without_a_match_keeps_the_palette_open() {
        let mut palette = Palette::new(sample(), "");
        typed(&mut palette, "zzz");
        assert!(palette.matches().is_empty());
        assert_eq!(press(&mut palette, KeyCode::Enter), PaletteOutcome::Open);
    }

    #[test]
    fn new_entries_keep_the_query() {
        let mut palette = Palette::new(Vec::new(), "mig");
        palette.set_entries(sample());
        assert_eq!(labels(&palette.matches())[0], "run migrate-up");
    }

    fn prompt_press(prompt: &mut ParamPrompt, code: KeyCode) -> PromptOutcome {
        prompt.handle(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn prompt_type(prompt: &mut ParamPrompt, text: &str) {
        for c in text.chars() {
            prompt_press(prompt, KeyCode::Char(c));
        }
    }

    #[test]
    fn a_required_parameter_must_be_given() {
        let mut prompt = ParamPrompt::new(recipe(
            "migrate-new",
            vec![param("name", None, false, ParamKind::Singular)],
        ))
        .unwrap();
        assert_eq!(prompt.recipe().name, "migrate-new");
        assert_eq!(prompt.param().name, "name");
        assert_eq!(
            prompt_press(&mut prompt, KeyCode::Enter),
            PromptOutcome::Editing
        );
        assert_eq!(prompt.error(), Some("name is required"));
        prompt_type(&mut prompt, "add foo");
        assert_eq!(
            prompt_press(&mut prompt, KeyCode::Enter),
            PromptOutcome::Done(vec!["add foo".into()])
        );
    }

    #[test]
    fn defaults_are_prefilled_and_variadic_answers_split() {
        let mut prompt = ParamPrompt::new(recipe(
            "import-staging",
            vec![
                param("file", Some("staging.sql"), true, ParamKind::Singular),
                param("ARGS", None, true, ParamKind::Star),
            ],
        ))
        .unwrap();
        assert_eq!(prompt.input().text(), "staging.sql");
        assert_eq!(
            prompt_press(&mut prompt, KeyCode::Enter),
            PromptOutcome::Editing
        );
        assert_eq!(prompt.param().name, "ARGS");
        prompt_type(&mut prompt, "--yes  --dry-run");
        assert_eq!(
            prompt_press(&mut prompt, KeyCode::Enter),
            PromptOutcome::Done(vec![
                "staging.sql".into(),
                "--yes".into(),
                "--dry-run".into()
            ])
        );
    }

    #[test]
    fn an_empty_optional_answer_ends_the_arguments() {
        let mut prompt = ParamPrompt::new(recipe(
            "bench",
            vec![
                param("out", Some("target"), true, ParamKind::Singular),
                param("history", Some("730"), true, ParamKind::Singular),
            ],
        ))
        .unwrap();
        prompt.handle(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL));
        assert_eq!(
            prompt_press(&mut prompt, KeyCode::Enter),
            PromptOutcome::Done(Vec::new())
        );
    }

    #[test]
    fn plus_parameters_need_a_word_and_escape_cancels() {
        let mut prompt = ParamPrompt::new(recipe(
            "x",
            vec![param("FILES", None, false, ParamKind::Plus)],
        ))
        .unwrap();
        assert_eq!(
            prompt_press(&mut prompt, KeyCode::Enter),
            PromptOutcome::Editing
        );
        assert_eq!(prompt.error(), Some("FILES is required"));
        assert_eq!(
            prompt_press(&mut prompt, KeyCode::Esc),
            PromptOutcome::Cancelled
        );
        assert!(ParamPrompt::new(recipe("migrate-up", Vec::new())).is_none());
    }
}
