use std::collections::HashMap;

use async_trait::async_trait;
use bollard::{Docker, query_parameters::ListContainersOptionsBuilder};

use super::{MANUAL_MIGRATION, migrate_fix};
use crate::{
    doctor::{Check, Env, Finding, Fix, Severity, Trigger},
    units::migrations::{applied, migration_versions, missing},
};

pub fn behind(pending: &[i64]) -> Vec<Finding> {
    let Some(newest) = pending.last() else {
        return Vec::new();
    };
    vec![
        Finding::new(
            "dev-db-migrations",
            Severity::Error,
            format!("dev database is {} migration(s) behind", pending.len()),
        )
        .detail(format!(
            "Newest missing migration: {newest}.\n{MANUAL_MIGRATION}"
        ))
        .fix(migrate_fix()),
    ]
}

pub struct DevDbMigrations;

#[async_trait]
impl Check for DevDbMigrations {
    fn id(&self) -> &'static str {
        "dev-db-migrations"
    }

    async fn diagnose(&self, env: &Env) -> Result<Vec<Finding>, String> {
        let expected = migration_versions(&env.root.join("backend/migrations"))
            .map_err(|err| err.to_string())?;
        let user = env
            .vars
            .get("postgres_user")
            .map_err(|err| err.to_string())?;
        let database = env.vars.get("postgres_db").map_err(|err| err.to_string())?;
        let applied = applied(&env.compose, &user, &database)
            .await
            .map_err(|err| format!("cannot read the applied migrations: {err}"))?;
        Ok(behind(&missing(&expected, &applied)))
    }

    fn reacts_to(&self, trigger: &Trigger<'_>) -> bool {
        match trigger {
            Trigger::Log(record) => {
                record.message.contains("does not exist")
                    && (record.message.contains("column \"")
                        || record.message.contains("relation \""))
            }
            Trigger::Failed { unit } => *unit == "migrations",
        }
    }
}

/// The label `just test-clean` removes containers by.
const TESTCONTAINERS_LABEL: &str = "org.testcontainers.managed-by=testcontainers";

pub fn leftovers(count: usize) -> Vec<Finding> {
    if count == 0 {
        return Vec::new();
    }
    vec![
        Finding::new(
            "testcontainers",
            Severity::Hint,
            format!("{count} testcontainer(s) left over from earlier test runs"),
        )
        .detail(
            "Aborted integration test runs leave their Postgres containers behind; they keep \
             holding memory and ports.",
        )
        .fix(Fix::just("test-clean").destructive()),
    ]
}

pub struct Testcontainers;

#[async_trait]
impl Check for Testcontainers {
    fn id(&self) -> &'static str {
        "testcontainers"
    }

    async fn diagnose(&self, _env: &Env) -> Result<Vec<Finding>, String> {
        let docker = Docker::connect_with_local_defaults().map_err(|err| err.to_string())?;
        let filters = HashMap::from([("label", vec![TESTCONTAINERS_LABEL.to_owned()])]);
        let options = ListContainersOptionsBuilder::new()
            .all(true)
            .filters(&filters)
            .build();
        let containers = docker
            .list_containers(Some(options))
            .await
            .map_err(|err| err.to_string())?;
        Ok(leftovers(containers.len()))
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::{
        config::LogFormat,
        doctor::{Severity, Trigger},
        logs::parse_line,
    };

    #[test]
    fn leftover_testcontainers_are_removed_only_after_confirmation() {
        assert!(leftovers(0).is_empty());
        let findings = leftovers(3);
        assert_eq!(
            findings[0].title,
            "3 testcontainer(s) left over from earlier test runs"
        );
        assert_eq!(findings[0].severity, Severity::Hint);
        let fix = findings[0].fix.as_ref().unwrap();
        assert_eq!(fix.command, "just test-clean");
        assert!(fix.destructive);
    }

    #[test]
    fn a_dev_database_behind_is_fixed_by_rebuilding_the_migrate_binary() {
        assert!(behind(&[]).is_empty());
        let findings = behind(&[20260101000000, 20260202000000]);
        assert_eq!(findings[0].title, "dev database is 2 migration(s) behind");
        assert_eq!(findings[0].severity, Severity::Error);
        assert!(findings[0].detail.contains("20260202000000"));
        let fix = findings[0].fix.as_ref().unwrap();
        assert_eq!(
            fix.command,
            "touch backend/crates/app/src/migrations.rs && just migrate-up"
        );
        assert_eq!(fix.cwd, "");
        assert!(!fix.destructive);
    }

    #[test]
    fn missing_columns_and_a_failed_migration_trigger_the_check() {
        let line = |message: &str| parse_line("backend", LogFormat::Plain, message, Utc::now());
        let column =
            line(r#"ERROR error returned from database: column "watered_at" does not exist"#);
        let relation = line(r#"relation "plugins" does not exist"#);
        let other = line("file does not exist");

        assert!(DevDbMigrations.reacts_to(&Trigger::Log(&column)));
        assert!(DevDbMigrations.reacts_to(&Trigger::Log(&relation)));
        assert!(!DevDbMigrations.reacts_to(&Trigger::Log(&other)));
        assert!(DevDbMigrations.reacts_to(&Trigger::Failed { unit: "migrations" }));
        assert!(!DevDbMigrations.reacts_to(&Trigger::Failed { unit: "backend" }));
    }
}
