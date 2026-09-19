use sqlx::{Postgres, QueryBuilder};

/// Maps a sort field's SQL key to the expressions that order by it. A key can
/// carry several expressions when one column needs more than one comparison,
/// as tree numbers do: letter prefix first, digits numerically second.
pub type SortColumns = &'static [(&'static str, &'static [&'static str])];

/// Appends `ORDER BY` for `key`, always `NULLS LAST` and always followed by a
/// tiebreaker on the primary key, so a row cannot shift between pages when two
/// rows compare equal.
pub(crate) fn push_order_by(
    qb: &mut QueryBuilder<'_, Postgres>,
    key: &str,
    descending: bool,
    columns: SortColumns,
    tiebreaker: &str,
) {
    let exprs = match columns.iter().find(|(name, _)| *name == key) {
        Some((_, exprs)) => *exprs,
        None => {
            tracing::error!(
                sort.key = key,
                "sort key has no column mapping; falling back to the first column"
            );
            columns
                .first()
                .map(|(_, exprs)| *exprs)
                .unwrap_or(&[] as &[&str])
        }
    };

    qb.push(" ORDER BY ");
    for expr in exprs {
        qb.push(*expr)
            .push(if descending { " DESC" } else { " ASC" })
            .push(" NULLS LAST, ");
    }
    qb.push(tiebreaker).push(" ASC");
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::{Postgres, QueryBuilder};

    const COLUMNS: SortColumns = &[
        ("name", &["tc.name"]),
        (
            "number",
            &[
                r"substring(t.number from '^\D*')",
                r"NULLIF(substring(t.number from '\d+'), '')::numeric",
            ],
        ),
    ];

    fn rendered(key: &str, descending: bool) -> String {
        let mut qb: QueryBuilder<'_, Postgres> = QueryBuilder::new("");
        push_order_by(&mut qb, key, descending, COLUMNS, "t.id");
        qb.into_sql()
    }

    #[test]
    fn appends_direction_nulls_last_and_the_tiebreaker() {
        assert_eq!(
            rendered("name", false),
            " ORDER BY tc.name ASC NULLS LAST, t.id ASC"
        );
        assert_eq!(
            rendered("name", true),
            " ORDER BY tc.name DESC NULLS LAST, t.id ASC"
        );
    }

    #[test]
    fn a_multi_expression_column_sorts_every_part_the_same_way() {
        let sql = rendered("number", true);

        assert_eq!(sql.matches("DESC NULLS LAST").count(), 2);
        assert!(sql.ends_with(", t.id ASC"));
    }

    #[test]
    fn an_unknown_key_falls_back_to_the_first_column() {
        // Reaching this branch is a programming error, not a bad request: the
        // key comes from a closed enum. Falling back beats panicking in a
        // request path.
        assert_eq!(
            rendered("does-not-exist", false),
            " ORDER BY tc.name ASC NULLS LAST, t.id ASC"
        );
    }
}
