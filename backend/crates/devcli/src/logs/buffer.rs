use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Utc};
use tokio::sync::broadcast;

use super::{Filter, LogRecord, builds::Builds};
use crate::sync::lock;

pub struct LogStore {
    capacity: usize,
    inner: Mutex<Inner>,
    tx: broadcast::Sender<LogRecord>,
    builds: Builds,
}

#[derive(Default)]
struct Inner {
    next_seq: u64,
    per_unit: HashMap<String, VecDeque<LogRecord>>,
}

impl LogStore {
    pub fn new(capacity_per_unit: usize) -> Arc<Self> {
        let (tx, _) = broadcast::channel(4096);
        Arc::new(Self {
            capacity: capacity_per_unit.max(1),
            inner: Mutex::default(),
            tx,
            builds: Builds::default(),
        })
    }

    /// Cargo progress the units printed; it never becomes a log record.
    pub fn builds(&self) -> &Builds {
        &self.builds
    }

    /// Broadcasting under the lock keeps subscribers in sequence order.
    pub fn push(&self, mut record: LogRecord) -> u64 {
        let mut inner = lock(&self.inner);
        inner.next_seq += 1;
        record.seq = inner.next_seq;
        let queue = inner.per_unit.entry(record.unit.clone()).or_default();
        if queue.len() == self.capacity {
            queue.pop_front();
        }
        queue.push_back(record.clone());
        let seq = record.seq;
        let _ = self.tx.send(record);
        seq
    }

    /// The newest `tail` records of `units` (every unit when empty) that pass `filter`, oldest
    /// first.
    pub fn snapshot(
        &self,
        units: &[String],
        filter: &Filter,
        tail: usize,
        now: DateTime<Utc>,
    ) -> Vec<LogRecord> {
        let inner = lock(&self.inner);
        let mut merged: Vec<LogRecord> = inner
            .per_unit
            .iter()
            .filter(|(unit, _)| units.is_empty() || units.contains(unit))
            .flat_map(|(_, queue)| {
                queue
                    .iter()
                    .rev()
                    .filter(|record| filter.matches(record, now))
                    .take(tail)
                    .cloned()
            })
            .collect();
        merged.sort_by_key(|record| record.seq);
        let skip = merged.len().saturating_sub(tail);
        merged.split_off(skip)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<LogRecord> {
        self.tx.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::{
        config::LogFormat,
        logs::{Filter, parse_line},
    };

    fn record(unit: &str, message: &str) -> LogRecord {
        parse_line(unit, LogFormat::Plain, message, Utc::now())
    }

    fn messages(records: &[LogRecord]) -> Vec<&str> {
        records.iter().map(|r| r.message.as_str()).collect()
    }

    fn all() -> Filter {
        Filter::default()
    }

    #[test]
    fn drops_the_oldest_record_per_unit_at_capacity() {
        let store = LogStore::new(2);
        store.push(record("db", "a"));
        store.push(record("db", "b"));
        store.push(record("backend", "x"));
        store.push(record("db", "c"));

        assert_eq!(
            messages(&store.snapshot(&["db".into()], &all(), 10, Utc::now())),
            ["b", "c"]
        );
        assert_eq!(
            messages(&store.snapshot(&[], &all(), 10, Utc::now())),
            ["b", "x", "c"]
        );
    }

    #[test]
    fn snapshot_returns_the_newest_tail_in_sequence_order() {
        let store = LogStore::new(10);
        for message in ["1", "2", "3", "4"] {
            store.push(record(
                if message == "2" { "db" } else { "backend" },
                message,
            ));
        }
        assert_eq!(
            messages(&store.snapshot(&[], &all(), 3, Utc::now())),
            ["2", "3", "4"]
        );
    }

    #[test]
    fn the_tail_counts_only_records_that_pass_the_filter() {
        let store = LogStore::new(10);
        for message in ["ERROR a", "info", "ERROR b", "info", "info"] {
            store.push(record("db", message));
        }
        let errors: Filter = "level:error".parse().unwrap();
        assert_eq!(
            messages(&store.snapshot(&[], &errors, 2, Utc::now())),
            ["ERROR a", "ERROR b"]
        );
    }

    #[tokio::test]
    async fn subscribers_receive_records_with_sequence_numbers() {
        let store = LogStore::new(10);
        let mut rx = store.subscribe();
        let seq = store.push(record("db", "ready"));

        let received = rx.recv().await.unwrap();
        assert_eq!(received.seq, seq);
        assert_eq!(received.message, "ready");
    }
}
