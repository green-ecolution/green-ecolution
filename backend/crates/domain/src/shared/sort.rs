//! Sort direction, shared by every list query.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

impl SortDirection {
    pub fn is_descending(self) -> bool {
        matches!(self, Self::Descending)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_ascending() {
        assert_eq!(SortDirection::default(), SortDirection::Ascending);
        assert!(!SortDirection::default().is_descending());
    }
}
