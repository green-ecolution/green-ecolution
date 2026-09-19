//! Shared machinery for paginated list queries.
//!
//! One `ListSpec` describes a list; `fetch` turns it into up to three
//! statements: the count within the caller's scope, the count after the
//! caller's own filters, and the page itself. Predicates and sort columns are
//! written once and reused across all three, which is the duplication this
//! module exists to remove.
//!
//! **Invariant.** Query text is authored in this repository and never derived
//! from request input; where a fragment must carry a configured constant it is
//! composed once at startup, never per request. Values always travel through
//! `push_bind`. These queries are assembled at runtime and are therefore not
//! covered by sqlx's compile-time checks; that split is what keeps them safe,
//! and the integration tests are what keep the column names honest.

pub mod order;
pub mod predicate;

use domain::{
    RepositoryError,
    shared::pagination::{Page, Pagination, SearchPage},
};
use sqlx::{PgPool, Postgres, QueryBuilder, postgres::PgRow};

pub use order::SortColumns;
pub use predicate::{ArchiveFilter, Predicate};

use order::push_order_by;

pub struct ListSpec<'a> {
    base: &'a str,
    primary_key: &'a str,
    filter_joins: Vec<&'a str>,
    projection_joins: Vec<&'a str>,
    group_by: Option<&'a str>,
    scope: Vec<Predicate>,
    filters: Vec<Predicate>,
    sort: Option<(&'a str, bool, SortColumns)>,
    pagination: Pagination,
}

impl<'a> ListSpec<'a> {
    /// `base` is the `FROM` clause including its alias, `primary_key` the
    /// aliased primary key used for counting and as the sort tiebreaker.
    pub fn new(base: &'a str, primary_key: &'a str) -> Self {
        Self {
            base,
            primary_key,
            filter_joins: Vec::new(),
            projection_joins: Vec::new(),
            group_by: None,
            scope: Vec::new(),
            filters: Vec::new(),
            sort: None,
            pagination: Pagination::default(),
        }
    }

    /// A join a filter or a sort expression depends on. Counts include it, and
    /// they switch to `COUNT(DISTINCT pk)` because such a join can multiply
    /// rows. A join that can multiply rows must be paired with a `group_by` on
    /// the primary key, or the page will return more rows than `total`
    /// reports — the builder cannot add `DISTINCT` itself without breaking
    /// aggregate projections.
    pub fn join(mut self, join: &'a str) -> Self {
        self.filter_joins.push(join);
        self
    }

    /// A join only the projection needs, typically to aggregate children. The
    /// counts leave it out so they count records and not join rows.
    pub fn projection_join(mut self, join: &'a str) -> Self {
        self.projection_joins.push(join);
        self
    }

    pub fn group_by(mut self, group_by: &'a str) -> Self {
        self.group_by = Some(group_by);
        self
    }

    /// A restriction that is not the caller's own filter: visibility, provider,
    /// owning organization. It applies to `total_unfiltered` as well.
    pub fn scope(mut self, predicate: Option<Predicate>) -> Self {
        self.scope.extend(predicate);
        self
    }

    pub fn filter(mut self, predicate: Option<Predicate>) -> Self {
        self.filters.extend(predicate);
        self
    }

    pub fn sort(mut self, key: &'a str, descending: bool, columns: SortColumns) -> Self {
        self.sort = Some((key, descending, columns));
        self
    }

    pub fn page(mut self, pagination: Pagination) -> Self {
        self.pagination = pagination;
        self
    }

    fn has_filters(&self) -> bool {
        !self.filters.is_empty()
    }

    fn push_where(&self, qb: &mut QueryBuilder<'a, Postgres>, with_filters: bool) {
        let predicates = self
            .scope
            .iter()
            .chain(
                self.filters
                    .iter()
                    .take(if with_filters { self.filters.len() } else { 0 }),
            );

        let mut first = true;
        for predicate in predicates {
            qb.push(if first { " WHERE " } else { " AND " });
            predicate.push(qb);
            first = false;
        }
    }

    fn count_builder(&self, with_filters: bool) -> QueryBuilder<'a, Postgres> {
        let mut qb: QueryBuilder<'a, Postgres> = QueryBuilder::new("SELECT ");
        if self.filter_joins.is_empty() {
            qb.push("COUNT(*)");
        } else {
            qb.push("COUNT(DISTINCT ").push(self.primary_key).push(")");
        }
        qb.push(" FROM ").push(self.base);
        for join in &self.filter_joins {
            qb.push(" ").push(*join);
        }
        self.push_where(&mut qb, with_filters);
        qb
    }

    fn page_builder(&self, columns: &str) -> QueryBuilder<'a, Postgres> {
        let mut qb: QueryBuilder<'a, Postgres> = QueryBuilder::new("SELECT ");
        qb.push(columns.to_owned()).push(" FROM ").push(self.base);
        for join in self.filter_joins.iter().chain(self.projection_joins.iter()) {
            qb.push(" ").push(*join);
        }
        self.push_where(&mut qb, true);
        if let Some(group_by) = self.group_by {
            qb.push(" GROUP BY ").push(group_by);
        }
        if let Some((key, descending, columns)) = self.sort {
            push_order_by(&mut qb, key, descending, columns, self.primary_key);
        } else {
            qb.push(" ORDER BY ").push(self.primary_key).push(" ASC");
        }
        qb.push(" LIMIT ")
            .push_bind(i64::try_from(self.pagination.limit()).unwrap_or(i64::MAX))
            .push(" OFFSET ")
            .push_bind(i64::try_from(self.pagination.offset()).unwrap_or(i64::MAX));
        qb
    }

    #[cfg(test)]
    fn count_sql(&self, with_filters: bool) -> String {
        self.count_builder(with_filters).into_sql()
    }

    #[cfg(test)]
    fn page_sql(&self, columns: &str) -> String {
        self.page_builder(columns).into_sql()
    }

    /// Runs the list. `columns` is the projection, written exactly as the row
    /// type's `FromRow` expects its fields to be named.
    #[tracing::instrument(level = "trace", skip_all, fields(list.base = self.base))]
    pub async fn fetch<T>(
        self,
        pool: &PgPool,
        columns: &str,
    ) -> Result<SearchPage<T>, RepositoryError>
    where
        T: for<'r> sqlx::FromRow<'r, PgRow> + Send + Unpin,
    {
        let total: i64 = self
            .count_builder(true)
            .build_query_scalar()
            .fetch_one(pool)
            .await?;

        // Without a filter the two counts ask the same question, so the second
        // statement is pure cost.
        let total_unfiltered: i64 = if self.has_filters() {
            self.count_builder(false)
                .build_query_scalar()
                .fetch_one(pool)
                .await?
        } else {
            total
        };

        let mut page_builder = self.page_builder(columns);
        tracing::trace!(sql = page_builder.sql(), "list page query");
        let items: Vec<T> = page_builder.build_query_as().fetch_all(pool).await?;

        Ok(SearchPage {
            page: Page {
                items,
                total: total as u64,
            },
            total_unfiltered: total_unfiltered as u64,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::shared::pagination::Pagination;

    const COLUMNS: SortColumns = &[("model", &["v.model"])];

    fn spec() -> ListSpec<'static> {
        ListSpec::new("vehicles v", "v.id")
            .scope(Predicate::any_of(
                "v.organization_id",
                vec![uuid::Uuid::nil()],
            ))
            .filter(Predicate::text_search(&["v.model"], Some("MAN".into())))
            .sort("model", false, COLUMNS)
            .page(Pagination::new(2, 25))
    }

    #[test]
    fn the_page_query_carries_filters_order_and_the_window() {
        let sql = spec().page_sql("v.id, v.model");

        assert!(sql.starts_with("SELECT v.id, v.model FROM vehicles v"));
        assert!(sql.contains("WHERE "));
        assert_eq!(sql.matches(" AND ").count(), 1);
        assert!(sql.contains("ORDER BY v.model ASC NULLS LAST, v.id ASC"));
        assert!(sql.contains("LIMIT "));
        assert!(sql.contains("OFFSET "));
    }

    #[test]
    fn the_unfiltered_count_drops_the_filters_but_keeps_the_scope() {
        let sql = spec().count_sql(false);

        assert!(sql.contains("v.organization_id"));
        assert!(!sql.contains("ILIKE"));
        assert!(!sql.contains("ORDER BY"));
        assert!(!sql.contains("LIMIT"));
    }

    #[test]
    fn counts_leave_out_projection_only_joins() {
        let sql = ListSpec::new("tree_clusters tc", "tc.id")
            .projection_join("LEFT JOIN trees t ON t.tree_cluster_id = tc.id")
            .group_by("tc.id")
            .sort("model", false, COLUMNS)
            .page(Pagination::default())
            .count_sql(true);

        assert!(!sql.contains("LEFT JOIN trees"));
        assert!(!sql.contains("GROUP BY"));
    }

    #[test]
    fn a_filtering_join_forces_a_distinct_count() {
        let sql = ListSpec::new("tree_clusters tc", "tc.id")
            .join("LEFT JOIN trees t ON t.tree_cluster_id = tc.id")
            .sort("model", false, COLUMNS)
            .page(Pagination::default())
            .count_sql(true);

        assert!(sql.contains("COUNT(DISTINCT tc.id)"));
        assert!(sql.contains("LEFT JOIN trees"));
    }

    #[test]
    fn without_a_filter_there_is_nothing_to_count_twice() {
        let spec = ListSpec::new("regions r", "r.id")
            .sort("model", false, COLUMNS)
            .page(Pagination::default());

        assert!(!spec.has_filters());
    }

    #[test]
    fn limit_and_offset_are_bound_not_formatted() {
        let sql = spec().page_sql("v.id, v.model");

        assert!(sql.contains("LIMIT $"), "limit must be bound: {sql}");
        assert!(sql.contains("OFFSET $"), "offset must be bound: {sql}");
    }

    #[test]
    fn no_sort_still_orders_by_the_primary_key() {
        let sql = ListSpec::new("regions r", "r.id")
            .page(Pagination::default())
            .page_sql("r.id");

        assert!(sql.ends_with("ORDER BY r.id ASC LIMIT $1 OFFSET $2"));
    }
}
