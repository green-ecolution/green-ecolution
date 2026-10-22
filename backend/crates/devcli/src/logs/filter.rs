use std::{fmt, str::FromStr};

use chrono::{DateTime, TimeDelta, Utc};

use super::{Level, LogRecord};

/// Terms are combined with AND; the comma-separated values inside one term with OR.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filter {
    terms: Vec<Term>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Term {
    LevelCmp(Cmp, Level),
    Levels(Vec<Level>),
    Units(Vec<String>),
    Targets(Vec<String>),
    Requests(Vec<String>),
    Since(u64),
    /// Stored lower-cased; matched case-insensitively.
    Text(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cmp {
    Ge,
    Gt,
    Le,
    Lt,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FilterError {
    #[error("unterminated quote")]
    UnterminatedQuote,
    #[error("empty search text")]
    EmptyText,
    #[error("unknown filter key `{0}`; quote text that contains a colon")]
    UnknownKey(String),
    #[error("`{0}` needs a value")]
    MissingValue(String),
    #[error("unknown level `{0}`; use trace, debug, info, warn or error")]
    UnknownLevel(String),
    #[error("invalid duration `{0}`; use e.g. 30s, 5m, 2h or 1d")]
    InvalidDuration(String),
}

impl Filter {
    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }

    pub fn matches(&self, record: &LogRecord, now: DateTime<Utc>) -> bool {
        self.terms.iter().all(|term| term.matches(record, now))
    }
}

impl Term {
    fn matches(&self, record: &LogRecord, now: DateTime<Utc>) -> bool {
        match self {
            Self::LevelCmp(cmp, level) => {
                record.level != Level::Unknown
                    && match cmp {
                        Cmp::Ge => record.level >= *level,
                        Cmp::Gt => record.level > *level,
                        Cmp::Le => record.level <= *level,
                        Cmp::Lt => record.level < *level,
                    }
            }
            Self::Levels(levels) => levels.contains(&record.level),
            Self::Units(units) => units.contains(&record.unit),
            Self::Targets(prefixes) => starts_with_any(record.target.as_deref(), prefixes),
            Self::Requests(prefixes) => starts_with_any(record.request_id.as_deref(), prefixes),
            Self::Since(secs) => i64::try_from(*secs)
                .ok()
                .and_then(TimeDelta::try_seconds)
                .and_then(|age| now.checked_sub_signed(age))
                .is_none_or(|cutoff| record.timestamp >= cutoff),
            Self::Text(needle) => record.message.to_lowercase().contains(needle.as_str()),
        }
    }
}

fn starts_with_any(value: Option<&str>, prefixes: &[String]) -> bool {
    value.is_some_and(|value| {
        prefixes
            .iter()
            .any(|prefix| value.starts_with(prefix.as_str()))
    })
}

impl FromStr for Filter {
    type Err = FilterError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut terms = Vec::new();
        let mut rest = input.trim_start();
        while !rest.is_empty() {
            let (term, tail) = match rest.strip_prefix('"') {
                Some(quoted) => {
                    let end = quoted.find('"').ok_or(FilterError::UnterminatedQuote)?;
                    (text_term(&quoted[..end])?, &quoted[end + 1..])
                }
                None => {
                    let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
                    (word_term(&rest[..end])?, &rest[end..])
                }
            };
            terms.push(term);
            rest = tail.trim_start();
        }
        Ok(Self { terms })
    }
}

fn text_term(text: &str) -> Result<Term, FilterError> {
    if text.is_empty() {
        return Err(FilterError::EmptyText);
    }
    Ok(Term::Text(text.to_lowercase()))
}

fn word_term(word: &str) -> Result<Term, FilterError> {
    if let Some(rest) = word.strip_prefix("level") {
        for (symbol, cmp) in [
            (">=", Cmp::Ge),
            ("<=", Cmp::Le),
            (">", Cmp::Gt),
            ("<", Cmp::Lt),
        ] {
            if let Some(value) = rest.strip_prefix(symbol) {
                return Ok(Term::LevelCmp(cmp, parse_level(value)?));
            }
        }
        if let Some(values) = rest.strip_prefix(':').or_else(|| rest.strip_prefix('=')) {
            let levels = list("level", values)?
                .into_iter()
                .map(parse_level)
                .collect::<Result<_, _>>()?;
            return Ok(Term::Levels(levels));
        }
    }
    let Some((key, values)) = word.split_once(':') else {
        return text_term(word);
    };
    let owned = |values: Vec<&str>| values.into_iter().map(str::to_owned).collect();
    match key {
        "unit" => Ok(Term::Units(owned(list(key, values)?))),
        "target" => Ok(Term::Targets(owned(list(key, values)?))),
        "req" => Ok(Term::Requests(owned(list(key, values)?))),
        "since" => parse_duration(values).map(Term::Since),
        _ => Err(FilterError::UnknownKey(key.to_owned())),
    }
}

fn list<'a>(key: &str, values: &'a str) -> Result<Vec<&'a str>, FilterError> {
    let items: Vec<&str> = values
        .split(',')
        .filter(|value| !value.is_empty())
        .collect();
    if items.is_empty() {
        return Err(FilterError::MissingValue(key.to_owned()));
    }
    Ok(items)
}

fn parse_level(value: &str) -> Result<Level, FilterError> {
    match value.to_ascii_lowercase().as_str() {
        "" => Err(FilterError::MissingValue("level".to_owned())),
        "trace" => Ok(Level::Trace),
        "debug" => Ok(Level::Debug),
        "info" => Ok(Level::Info),
        "warn" | "warning" => Ok(Level::Warn),
        "error" => Ok(Level::Error),
        _ => Err(FilterError::UnknownLevel(value.to_owned())),
    }
}

fn parse_duration(value: &str) -> Result<u64, FilterError> {
    let invalid = || FilterError::InvalidDuration(value.to_owned());
    let split = value
        .len()
        .checked_sub(1)
        .filter(|&index| value.is_char_boundary(index))
        .ok_or_else(invalid)?;
    let (number, unit) = value.split_at(split);
    let factor = match unit {
        "s" => 1,
        "m" => 60,
        "h" => 3_600,
        "d" => 86_400,
        _ => return Err(invalid()),
    };
    number
        .parse::<u64>()
        .ok()
        .and_then(|number| number.checked_mul(factor))
        .ok_or_else(invalid)
}

impl fmt::Display for Filter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, term) in self.terms.iter().enumerate() {
            if index > 0 {
                f.write_str(" ")?;
            }
            match term {
                Term::LevelCmp(cmp, level) => {
                    let symbol = match cmp {
                        Cmp::Ge => ">=",
                        Cmp::Gt => ">",
                        Cmp::Le => "<=",
                        Cmp::Lt => "<",
                    };
                    write!(f, "level{symbol}{}", level_name(*level))?;
                }
                Term::Levels(levels) => {
                    let names: Vec<&str> = levels.iter().map(|level| level_name(*level)).collect();
                    write!(f, "level:{}", names.join(","))?;
                }
                Term::Units(units) => write!(f, "unit:{}", units.join(","))?,
                Term::Targets(targets) => write!(f, "target:{}", targets.join(","))?,
                Term::Requests(ids) => write!(f, "req:{}", ids.join(","))?,
                Term::Since(secs) => write!(f, "since:{}", duration_label(*secs))?,
                Term::Text(text) => write!(f, "\"{text}\"")?,
            }
        }
        Ok(())
    }
}

fn level_name(level: Level) -> &'static str {
    match level {
        Level::Unknown => "unknown",
        Level::Trace => "trace",
        Level::Debug => "debug",
        Level::Info => "info",
        Level::Warn => "warn",
        Level::Error => "error",
    }
}

fn duration_label(secs: u64) -> String {
    match secs {
        0 => "0s".to_owned(),
        secs if secs % 86_400 == 0 => format!("{}d", secs / 86_400),
        secs if secs % 3_600 == 0 => format!("{}h", secs / 3_600),
        secs if secs % 60 == 0 => format!("{}m", secs / 60),
        secs => format!("{secs}s"),
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, TimeDelta, Utc};
    use quickcheck::{Arbitrary, Gen};
    use quickcheck_macros::quickcheck;

    use super::*;
    use crate::{config::LogFormat, logs::parse_line};

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-10-22T13:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn backend(
        level: &str,
        target: &str,
        message: &str,
        request_id: &str,
        age_secs: i64,
    ) -> LogRecord {
        let timestamp = (now() - TimeDelta::seconds(age_secs)).to_rfc3339();
        let line = format!(
            r#"{{"timestamp":"{timestamp}","level":"{level}","target":"{target}","message":"{message}","request_id":"{request_id}"}}"#
        );
        parse_line("backend", LogFormat::TracingJson, &line, now())
    }

    fn plain(unit: &str, message: &str) -> LogRecord {
        parse_line(unit, LogFormat::Plain, message, now())
    }

    fn filter(text: &str) -> Filter {
        text.parse().unwrap()
    }

    #[test]
    fn the_spec_example_combines_terms_with_and() {
        let f = filter(r#"level>=warn unit:backend,db target:sqlx "jwks""#);
        assert!(f.matches(
            &backend("WARN", "sqlx::query", "JWKS refresh slow", "r1", 0),
            now()
        ));
        assert!(!f.matches(
            &backend("INFO", "sqlx::query", "jwks refresh slow", "r1", 0),
            now()
        ));
        assert!(!f.matches(
            &backend("WARN", "server::http", "jwks refresh slow", "r1", 0),
            now()
        ));
        assert!(!f.matches(
            &backend("WARN", "sqlx::query", "pool closed", "r1", 0),
            now()
        ));
    }

    #[test]
    fn level_comparisons_never_match_records_without_a_level() {
        let unknown = plain("frontend", "VITE ready");
        assert!(!filter("level<=info").matches(&unknown, now()));
        assert!(!filter("level>=trace").matches(&unknown, now()));
        assert!(filter("level<warn").matches(&backend("DEBUG", "t", "m", "r", 0), now()));
        assert!(filter("level:debug,error").matches(&backend("ERROR", "t", "m", "r", 0), now()));
        assert!(filter("level=WARNING").matches(&backend("WARN", "t", "m", "r", 0), now()));
    }

    #[test]
    fn units_are_exact_and_targets_and_requests_are_prefixes() {
        assert!(filter("unit:db").matches(&plain("db", "x"), now()));
        assert!(!filter("unit:d").matches(&plain("db", "x"), now()));
        assert!(filter("req:0192").matches(&backend("INFO", "t", "m", "0192f3aa", 0), now()));
        assert!(!filter("req:0193").matches(&backend("INFO", "t", "m", "0192f3aa", 0), now()));
        assert!(!filter("req:0192").matches(&plain("db", "0192"), now()));
        assert!(!filter("target:sqlx").matches(&plain("db", "sqlx"), now()));
    }

    #[test]
    fn since_keeps_only_recent_records() {
        let f = filter("since:5m");
        assert!(f.matches(&backend("INFO", "t", "m", "r", 299), now()));
        assert!(!f.matches(&backend("INFO", "t", "m", "r", 301), now()));
    }

    #[test]
    fn text_is_case_insensitive_and_may_be_bare() {
        assert!(filter("Ready").matches(&plain("db", "database system is ready"), now()));
        assert!(filter(r#""system is""#).matches(&plain("db", "database system is ready"), now()));
        assert!(
            !filter(r#""system  is""#).matches(&plain("db", "database system is ready"), now())
        );
    }

    #[test]
    fn the_empty_filter_matches_everything() {
        let f = filter("   ");
        assert!(f.is_empty());
        assert_eq!(f, Filter::default());
        assert!(f.matches(&plain("db", "x"), now()));
    }

    #[test]
    fn reports_malformed_filters() {
        let err = |text: &str| text.parse::<Filter>().unwrap_err();
        assert_eq!(err(r#""open"#), FilterError::UnterminatedQuote);
        assert_eq!(err(r#""""#), FilterError::EmptyText);
        assert_eq!(err("http://x"), FilterError::UnknownKey("http".into()));
        assert_eq!(err("unit:"), FilterError::MissingValue("unit".into()));
        assert_eq!(err("level>="), FilterError::MissingValue("level".into()));
        assert_eq!(err("level:loud"), FilterError::UnknownLevel("loud".into()));
        assert_eq!(err("since:5y"), FilterError::InvalidDuration("5y".into()));
        assert_eq!(err("since:m"), FilterError::InvalidDuration("m".into()));
    }

    #[test]
    fn displays_the_canonical_form() {
        let f =
            filter(r#"level=WARNING unit:db,backend since:120s req:01 target:sqlx "Pool Closed""#);
        assert_eq!(
            f.to_string(),
            r#"level:warn unit:db,backend since:2m req:01 target:sqlx "pool closed""#
        );
    }

    const LEVELS: [Level; 5] = [
        Level::Trace,
        Level::Debug,
        Level::Info,
        Level::Warn,
        Level::Error,
    ];
    const CMPS: [Cmp; 4] = [Cmp::Ge, Cmp::Gt, Cmp::Le, Cmp::Lt];

    fn chars(g: &mut Gen, alphabet: &[char], max: usize) -> String {
        let len = usize::arbitrary(g) % max + 1;
        (0..len).map(|_| *g.choose(alphabet).unwrap()).collect()
    }

    fn word(g: &mut Gen) -> String {
        chars(g, &['a', 'z', '0', '9', '-', '_', '.', ':', '/'], 8)
    }

    fn some<T>(g: &mut Gen, item: impl Fn(&mut Gen) -> T) -> Vec<T> {
        let len = usize::arbitrary(g) % 3 + 1;
        (0..len).map(|_| item(g)).collect()
    }

    impl Arbitrary for Filter {
        fn arbitrary(g: &mut Gen) -> Self {
            let count = usize::arbitrary(g) % 4;
            let terms = (0..count)
                .map(|_| match u8::arbitrary(g) % 7 {
                    0 => Term::LevelCmp(*g.choose(&CMPS).unwrap(), *g.choose(&LEVELS).unwrap()),
                    1 => Term::Levels(some(g, |g| *g.choose(&LEVELS).unwrap())),
                    2 => Term::Units(some(g, word)),
                    3 => Term::Targets(some(g, word)),
                    4 => Term::Requests(some(g, word)),
                    5 => Term::Since(u64::from(u32::arbitrary(g))),
                    _ => Term::Text(chars(g, &['a', 'z', '0', ' ', 'ü', '-', ':'], 12)),
                })
                .collect();
            Self { terms }
        }
    }

    #[quickcheck]
    fn display_then_parse_round_trips(filter: Filter) -> bool {
        filter.to_string().parse::<Filter>() == Ok(filter)
    }

    #[quickcheck]
    fn parsing_arbitrary_input_never_panics(input: String) -> bool {
        let _ = input.parse::<Filter>();
        true
    }
}
