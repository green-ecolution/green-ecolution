//! Query-plan capture.
//!
//! A plan often says more than a timing does: a sequential scan still present
//! at the largest scale is a finding regardless of how many milliseconds it
//! happened to take on this machine.

use std::path::Path;

use sqlx::PgPool;

type Failure = Box<dyn std::error::Error>;

pub async fn capture(pool: &PgPool, name: &str, sql: &str, out_dir: &Path) -> Result<(), Failure> {
    let plans_dir = out_dir.join("plans");
    std::fs::create_dir_all(&plans_dir)?;

    // The probes carry their filters as literals, so there is nothing to bind
    // and the planner sees the same constants a real request produces.
    let statement = format!("EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON) {sql}");
    let plan: serde_json::Value = sqlx::query_scalar(&statement).fetch_one(pool).await?;

    std::fs::write(
        plans_dir.join(format!("{name}.json")),
        serde_json::to_vec_pretty(&plan)?,
    )?;
    Ok(())
}

pub fn has_seq_scan_on(plan: &serde_json::Value, table: &str) -> bool {
    fn walk(node: &serde_json::Value, table: &str) -> bool {
        let is_seq_scan = node.get("Node Type").and_then(|v| v.as_str()) == Some("Seq Scan")
            && node.get("Relation Name").and_then(|v| v.as_str()) == Some(table);
        if is_seq_scan {
            return true;
        }
        match node {
            serde_json::Value::Array(items) => items.iter().any(|i| walk(i, table)),
            serde_json::Value::Object(map) => map.values().any(|v| walk(v, table)),
            _ => false,
        }
    }
    walk(plan, table)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_a_sequential_scan_on_the_named_table() {
        let plan: serde_json::Value = serde_json::json!([{
            "Plan": {
                "Node Type": "Limit",
                "Plans": [{
                    "Node Type": "Seq Scan",
                    "Relation Name": "trees"
                }]
            }
        }]);

        assert!(has_seq_scan_on(&plan, "trees"));
        assert!(!has_seq_scan_on(&plan, "tree_clusters"));
    }

    #[test]
    fn an_index_scan_is_not_a_sequential_scan() {
        let plan: serde_json::Value = serde_json::json!([{
            "Plan": { "Node Type": "Index Scan", "Relation Name": "trees" }
        }]);

        assert!(!has_seq_scan_on(&plan, "trees"));
    }

    #[test]
    fn a_scan_nested_several_levels_down_is_still_found() {
        let plan: serde_json::Value = serde_json::json!([{
            "Plan": {
                "Node Type": "Aggregate",
                "Plans": [{
                    "Node Type": "Nested Loop",
                    "Plans": [
                        { "Node Type": "Index Scan", "Relation Name": "tree_clusters" },
                        { "Node Type": "Seq Scan", "Relation Name": "trees" }
                    ]
                }]
            }
        }]);

        assert!(has_seq_scan_on(&plan, "trees"));
    }
}
