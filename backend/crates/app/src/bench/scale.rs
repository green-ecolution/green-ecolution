/// One step of the benchmark's data volume. The four dimensions grow together
/// because they do so in reality too; the reading time series gets its own
/// dial (`--history-days`) since it would otherwise dominate everything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scale {
    Xs,
    S,
    M,
    L,
}

impl Scale {
    /// Ascending; the campaign grows through them in order.
    pub const ALL: [Scale; 4] = [Scale::Xs, Scale::S, Scale::M, Scale::L];

    /// Not `FromStr`: the error is a CLI message, not a domain
    /// `ValidationError`, and this type never leaves the benchmark tooling.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "xs" => Ok(Scale::Xs),
            "s" => Ok(Scale::S),
            "m" => Ok(Scale::M),
            "l" => Ok(Scale::L),
            other => Err(format!(
                "unknown scale '{other}', expected one of xs, s, m, l"
            )),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Scale::Xs => "xs",
            Scale::S => "s",
            Scale::M => "m",
            Scale::L => "l",
        }
    }

    pub fn trees(self) -> i64 {
        match self {
            Scale::Xs => 1_000,
            Scale::S => 10_000,
            Scale::M => 100_000,
            Scale::L => 500_000,
        }
    }

    pub fn clusters(self) -> i64 {
        self.trees() / 50
    }

    pub fn sensors(self) -> i64 {
        self.trees() / 100
    }

    pub fn plans(self) -> i64 {
        match self {
            Scale::Xs => 50,
            Scale::S => 200,
            Scale::M => 1_000,
            Scale::L => 2_000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_is_ascending_by_tree_count() {
        let counts: Vec<i64> = Scale::ALL.iter().map(|s| s.trees()).collect();
        let mut sorted = counts.clone();
        sorted.sort_unstable();
        assert_eq!(counts, sorted);
    }

    #[test]
    fn names_round_trip() {
        for scale in Scale::ALL {
            assert_eq!(Scale::parse(scale.name()), Ok(scale));
        }
    }

    #[test]
    fn unknown_names_are_rejected_with_the_offending_value() {
        let err = Scale::parse("xxl").expect_err("xxl is not a scale");
        assert!(err.contains("xxl"), "got: {err}");
    }
}
