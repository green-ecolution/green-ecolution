use sqlx::{Encode, Postgres, QueryBuilder, Type};

use crate::infra::sql::like_escape;

/// How a list treats rows that carry an archive timestamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArchiveFilter {
    #[default]
    ActiveOnly,
    Include,
    ArchivedOnly,
}

type PushFn = Box<dyn Fn(&mut QueryBuilder<'_, Postgres>) + Send + Sync>;

/// One `WHERE` term. Values travel through `push_bind`; the expression text is
/// always a `&'static str` written in this repository, never caller input.
/// That split is what keeps query text free of anything a request supplied,
/// now that these queries no longer go through the checked macros.
pub struct Predicate {
    push: PushFn,
}

impl std::fmt::Debug for Predicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Predicate")
    }
}

impl Predicate {
    pub(crate) fn push(&self, qb: &mut QueryBuilder<'_, Postgres>) {
        (self.push)(qb)
    }

    fn new(push: impl Fn(&mut QueryBuilder<'_, Postgres>) + Send + Sync + 'static) -> Self {
        Self {
            push: Box::new(push),
        }
    }

    /// Case-insensitive `ILIKE` across several columns, OR-joined and
    /// bracketed. A blank or absent needle filters nothing.
    pub fn text_search(columns: &'static [&'static str], value: Option<String>) -> Option<Self> {
        let needle = value.as_deref().map(str::trim).filter(|s| !s.is_empty())?;
        let pattern = format!("%{}%", like_escape(needle));

        Some(Self::new(move |qb| {
            qb.push("(");
            for (index, column) in columns.iter().enumerate() {
                if index > 0 {
                    qb.push(" OR ");
                }
                qb.push(*column)
                    .push(" ILIKE ")
                    .push_bind(pattern.clone())
                    .push(r" ESCAPE '\'");
            }
            qb.push(")");
        }))
    }

    /// `expr = ANY($n)`. An empty selection means "no filter", not "nothing
    /// matches".
    pub fn any_of<T>(expr: &'static str, values: Vec<T>) -> Option<Self>
    where
        T: Clone + Send + Sync + 'static,
        Vec<T>: for<'q> Encode<'q, Postgres> + Type<Postgres>,
    {
        if values.is_empty() {
            return None;
        }

        Some(Self::new(move |qb| {
            qb.push(expr)
                .push(" = ANY(")
                .push_bind(values.clone())
                .push(")");
        }))
    }

    /// `None` writes nothing; `Some(ids)` writes `= ANY(ids)` even when `ids` is
    /// empty, which matches no row. The empty case is load-bearing: an
    /// authorization scope that resolves to no organizations must hide
    /// everything, not everything-is-visible.
    pub fn any_of_opt<T>(expr: &'static str, values: Option<Vec<T>>) -> Option<Self>
    where
        T: Clone + Send + Sync + 'static,
        Vec<T>: for<'q> Encode<'q, Postgres> + Type<Postgres>,
    {
        let values = values?;

        Some(Self::new(move |qb| {
            qb.push(expr)
                .push(" = ANY(")
                .push_bind(values.clone())
                .push(")");
        }))
    }

    pub fn equals<T>(expr: &'static str, value: Option<T>) -> Option<Self>
    where
        T: Clone + Send + Sync + 'static + for<'q> Encode<'q, Postgres> + Type<Postgres>,
    {
        let value = value?;
        Some(Self::new(move |qb| {
            qb.push(expr).push(" = ").push_bind(value.clone());
        }))
    }

    /// Answers "does this row have one at all", for filters of the shape
    /// "with sensor / without sensor".
    pub fn is_present(expr: &'static str, flag: Option<bool>) -> Option<Self> {
        let present = flag?;
        Some(Self::new(move |qb| {
            qb.push(expr)
                .push(if present { " IS NOT NULL" } else { " IS NULL" });
        }))
    }

    pub fn archived(expr: &'static str, filter: ArchiveFilter) -> Option<Self> {
        match filter {
            ArchiveFilter::Include => None,
            ArchiveFilter::ActiveOnly => Some(Self::new(move |qb| {
                qb.push(expr).push(" IS NULL");
            })),
            ArchiveFilter::ArchivedOnly => Some(Self::new(move |qb| {
                qb.push(expr).push(" IS NOT NULL");
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{Postgres, QueryBuilder};

    fn rendered(predicate: Option<Predicate>) -> String {
        let mut qb: QueryBuilder<'_, Postgres> = QueryBuilder::new("");
        predicate.expect("predicate must be present").push(&mut qb);
        qb.into_sql()
    }

    #[test]
    fn empty_collection_writes_nothing() {
        assert!(Predicate::any_of::<uuid::Uuid>("v.id", Vec::new()).is_none());
    }

    #[test]
    fn any_of_opt_none_writes_nothing() {
        assert!(Predicate::any_of_opt::<uuid::Uuid>("t.organization_id", None).is_none());
    }

    #[test]
    fn any_of_opt_some_empty_still_writes_a_predicate_that_matches_nothing() {
        let sql = rendered(Predicate::any_of_opt::<uuid::Uuid>(
            "t.organization_id",
            Some(Vec::new()),
        ));

        assert_eq!(sql, "t.organization_id = ANY($1)");
    }

    #[test]
    fn blank_search_writes_nothing() {
        assert!(Predicate::text_search(&["v.model"], Some("   ".into())).is_none());
        assert!(Predicate::text_search(&["v.model"], None).is_none());
    }

    #[test]
    fn search_brackets_the_or_chain() {
        let sql = rendered(Predicate::text_search(
            &["v.number_plate", "v.model"],
            Some("FL".into()),
        ));

        assert!(sql.starts_with('('), "must be bracketed: {sql}");
        assert!(sql.ends_with(')'), "must be bracketed: {sql}");
        assert_eq!(sql.matches(" OR ").count(), 1);
        assert_eq!(sql.matches("ILIKE").count(), 2);
        assert!(sql.contains(r"ESCAPE '\'"));
    }

    #[test]
    fn presence_flag_picks_the_null_check() {
        assert_eq!(
            rendered(Predicate::is_present("t.sensor_id", Some(true))),
            "t.sensor_id IS NOT NULL"
        );
        assert_eq!(
            rendered(Predicate::is_present("t.sensor_id", Some(false))),
            "t.sensor_id IS NULL"
        );
        assert!(Predicate::is_present("t.sensor_id", None).is_none());
    }

    #[test]
    fn archive_filter_covers_its_three_states() {
        assert_eq!(
            rendered(Predicate::archived(
                "v.archived_at",
                ArchiveFilter::ActiveOnly
            )),
            "v.archived_at IS NULL"
        );
        assert_eq!(
            rendered(Predicate::archived(
                "v.archived_at",
                ArchiveFilter::ArchivedOnly
            )),
            "v.archived_at IS NOT NULL"
        );
        assert!(Predicate::archived("v.archived_at", ArchiveFilter::Include).is_none());
    }
}
