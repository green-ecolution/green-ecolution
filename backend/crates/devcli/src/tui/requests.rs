use std::collections::HashSet;

use crate::logs::{
    LogRecord,
    trace::{RequestSummary, Trace},
};

pub const REQUEST_CAPACITY: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestOrder {
    Newest,
    Slowest,
}

#[derive(Debug)]
pub struct Requests {
    /// Arrival order, oldest first; kept apart from the log buffer so that busy units cannot
    /// push requests out of the list.
    seen: Vec<RequestSummary>,
    ids: HashSet<String>,
    pub order: RequestOrder,
    /// By id, so the selection stays on its request while new ones arrive.
    selected: Option<String>,
    pub trace: Option<Trace>,
    /// Index of the trace line at the top of the pane.
    pub trace_scroll: usize,
}

impl Default for Requests {
    fn default() -> Self {
        Self {
            seen: Vec::new(),
            ids: HashSet::new(),
            order: RequestOrder::Newest,
            selected: None,
            trace: None,
            trace_scroll: 0,
        }
    }
}

impl Requests {
    /// Returns whether the record added a request to the list.
    pub fn observe(&mut self, record: &LogRecord) -> bool {
        let Some(summary) = RequestSummary::from_record(record) else {
            return false;
        };
        if !self.ids.insert(summary.request_id.clone()) {
            return false;
        }
        if self.seen.len() == REQUEST_CAPACITY {
            let dropped = self.seen.remove(0);
            self.ids.remove(&dropped.request_id);
        }
        self.seen.push(summary);
        true
    }

    pub fn is_pinned(&self) -> bool {
        self.selected.is_some()
    }

    pub fn ordered(&self) -> Vec<&RequestSummary> {
        let mut list: Vec<&RequestSummary> = self.seen.iter().rev().collect();
        if self.order == RequestOrder::Slowest {
            list.sort_by_key(|request| std::cmp::Reverse(request.latency_ms));
        }
        list
    }

    pub fn selected_index(&self) -> Option<usize> {
        let ordered = self.ordered();
        let pinned = self
            .selected
            .as_ref()
            .and_then(|id| ordered.iter().position(|request| request.request_id == *id));
        pinned.or_else(|| (!ordered.is_empty()).then_some(0))
    }

    pub fn selected_id(&self) -> Option<String> {
        let index = self.selected_index()?;
        Some(self.ordered()[index].request_id.clone())
    }

    /// Pins the current selection by id and returns it, for opening the view.
    pub fn pin_selection(&mut self) -> Option<String> {
        self.selected = self.selected_id();
        self.selected.clone()
    }

    /// Returns the newly selected request when the selection moved.
    pub fn select(&mut self, down: bool) -> Option<String> {
        let current = self.selected_index()?;
        let id = {
            let ordered = self.ordered();
            let next = if down {
                (current + 1).min(ordered.len() - 1)
            } else {
                current.saturating_sub(1)
            };
            (next != current || self.selected.is_none()).then(|| ordered[next].request_id.clone())
        }?;
        self.selected = Some(id.clone());
        self.trace = None;
        self.trace_scroll = 0;
        Some(id)
    }

    pub fn toggle_order(&mut self) {
        self.selected = self.selected_id();
        self.order = match self.order {
            RequestOrder::Newest => RequestOrder::Slowest,
            RequestOrder::Slowest => RequestOrder::Newest,
        };
    }

    /// A trace that arrives after the selection moved on is dropped.
    pub fn set_trace(&mut self, trace: Trace) {
        if self.selected.as_deref() == Some(trace.request_id.as_str()) {
            self.trace = Some(trace);
            self.trace_scroll = 0;
        }
    }

    pub fn scroll_trace(&mut self, down: bool, lines: usize) {
        self.trace_scroll = if down {
            self.trace_scroll
                .saturating_add(lines)
                .min(self.last_trace_line())
        } else {
            self.trace_scroll.saturating_sub(lines)
        };
    }

    pub fn jump_trace(&mut self, top: bool) {
        self.trace_scroll = if top { 0 } else { self.last_trace_line() };
    }

    fn last_trace_line(&self) -> usize {
        self.trace
            .as_ref()
            .map_or(0, |trace| trace.lines.len().saturating_sub(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logs::trace::tests::finished_request;

    fn with(requests: &[(&str, u64)]) -> Requests {
        let mut state = Requests::default();
        for (id, latency) in requests {
            state.observe(&finished_request(id, "/api", 200, *latency));
        }
        state
    }

    fn ids(state: &Requests) -> Vec<&str> {
        state
            .ordered()
            .iter()
            .map(|r| r.request_id.as_str())
            .collect()
    }

    #[test]
    fn observe_reports_whether_it_added_a_request() {
        let mut state = Requests::default();
        assert!(state.observe(&finished_request("a", "/api", 200, 1)));
        assert!(!state.observe(&finished_request("a", "/api", 200, 1)));
        let plain = crate::logs::parse_line(
            "db",
            crate::config::LogFormat::Plain,
            "hello",
            chrono::Utc::now(),
        );
        assert!(!state.observe(&plain));
    }
    #[test]
    fn lists_each_request_once_newest_first() {
        let mut state = with(&[("a", 5), ("b", 30)]);
        state.observe(&finished_request("a", "/api", 200, 5));
        assert_eq!(ids(&state), ["b", "a"]);
    }

    #[test]
    fn sorts_by_duration_and_keeps_the_selection() {
        let mut state = with(&[("a", 50), ("b", 5), ("c", 20)]);
        assert_eq!(state.select(true), Some("b".to_owned()));
        state.toggle_order();
        assert_eq!(ids(&state), ["a", "c", "b"]);
        assert_eq!(state.selected_id().as_deref(), Some("b"));
    }

    #[test]
    fn the_selection_follows_its_request_when_new_ones_arrive() {
        let mut state = with(&[("a", 1), ("b", 1)]);
        assert_eq!(state.pin_selection(), Some("b".to_owned()));
        state.observe(&finished_request("c", "/api", 200, 1));
        assert_eq!(state.selected_index(), Some(1));
        assert_eq!(state.select(false), Some("c".to_owned()));
        assert_eq!(state.select(false), None);
    }

    #[test]
    fn forgets_the_oldest_request_at_capacity() {
        let mut state = Requests::default();
        for index in 0..=REQUEST_CAPACITY {
            state.observe(&finished_request(&index.to_string(), "/api", 200, 1));
        }
        assert_eq!(state.ordered().len(), REQUEST_CAPACITY);
        assert_eq!(state.ordered().last().unwrap().request_id, "1");
        state.observe(&finished_request("0", "/api", 200, 1));
        assert_eq!(state.ordered()[0].request_id, "0");
    }

    #[test]
    fn only_the_trace_of_the_selected_request_is_kept() {
        let mut state = with(&[("a", 1), ("b", 1)]);
        state.pin_selection();
        let records = [finished_request("a", "/api", 200, 1)];
        state.set_trace(Trace::build("a", &records));
        assert!(state.trace.is_none());

        let records = [finished_request("b", "/api", 200, 1)];
        state.set_trace(Trace::build("b", &records));
        assert_eq!(state.trace.as_ref().unwrap().request_id, "b");
        state.scroll_trace(true, 10);
        assert_eq!(state.trace_scroll, 1);
        state.jump_trace(true);
        assert_eq!(state.trace_scroll, 0);
    }
}
