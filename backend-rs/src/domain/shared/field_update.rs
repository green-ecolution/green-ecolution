/// Tri-state update marker for nullable fields in partial updates.
///
/// Disambiguates "leave the field as-is" from "set the field to NULL".
#[derive(Debug, Default, Clone)]
pub enum FieldUpdate<T> {
    /// Field is not part of this update — keep the current value.
    #[default]
    Unchanged,
    /// Set the field to NULL.
    Cleared,
    /// Set the field to this value.
    Set(T),
}

impl<T> FieldUpdate<T> {
    /// True if the update touches this field at all (`Cleared` or `Set`).
    pub fn is_change(&self) -> bool {
        !matches!(self, Self::Unchanged)
    }

    /// New value to write, if any. `None` means "no change OR clear" — use
    /// `is_change()` together with this to drive `CASE WHEN $flag THEN $value`
    /// patterns in SQL.
    pub fn as_set(&self) -> Option<&T> {
        match self {
            Self::Set(v) => Some(v),
            _ => None,
        }
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> FieldUpdate<U> {
        match self {
            Self::Unchanged => FieldUpdate::Unchanged,
            Self::Cleared => FieldUpdate::Cleared,
            Self::Set(v) => FieldUpdate::Set(f(v)),
        }
    }
}

impl<T> From<Option<T>> for FieldUpdate<T> {
    /// Convenience conversion: `Some(v)` → `Set(v)`, `None` → `Cleared`.
    /// Use only when you really mean "explicit clear on None".
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => Self::Set(v),
            None => Self::Cleared,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_unchanged() {
        let f: FieldUpdate<i32> = FieldUpdate::default();
        assert!(!f.is_change());
        assert!(f.as_set().is_none());
    }

    #[test]
    fn cleared_is_change_with_no_value() {
        let f: FieldUpdate<i32> = FieldUpdate::Cleared;
        assert!(f.is_change());
        assert!(f.as_set().is_none());
    }

    #[test]
    fn set_carries_value() {
        let f = FieldUpdate::Set(42);
        assert!(f.is_change());
        assert_eq!(f.as_set(), Some(&42));
    }

    #[test]
    fn map_preserves_state() {
        assert!(matches!(
            FieldUpdate::<i32>::Unchanged.map(|v| v + 1),
            FieldUpdate::Unchanged
        ));
        assert!(matches!(
            FieldUpdate::<i32>::Cleared.map(|v| v + 1),
            FieldUpdate::Cleared
        ));
        assert!(matches!(
            FieldUpdate::Set(1).map(|v| v + 1),
            FieldUpdate::Set(2)
        ));
    }
}
