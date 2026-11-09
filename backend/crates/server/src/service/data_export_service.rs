use std::sync::Arc;

use domain::{
    Id, RepositoryError,
    data_export::{
        BASE_URL_FIELD, ConnectionReport, ConnectionTester, ExportTargetConfig, ExportTargetDraft,
        ExportTargetReader, ExportTargetView, ExportTargetWriter, ExportUrlPolicy, TOKEN_URL_FIELD,
    },
    organization::Organization,
    settings::SettingsReader,
};

use super::ServiceError;

/// Configuration of an organization's export target. Permissions are checked
/// by the handlers, like for the other organization settings.
pub struct DataExportService {
    reader: Arc<dyn ExportTargetReader>,
    writer: Arc<dyn ExportTargetWriter>,
    tester: Arc<dyn ConnectionTester>,
    settings: Arc<dyn SettingsReader>,
    policy: ExportUrlPolicy,
}

impl DataExportService {
    pub fn new(
        reader: Arc<dyn ExportTargetReader>,
        writer: Arc<dyn ExportTargetWriter>,
        tester: Arc<dyn ConnectionTester>,
        settings: Arc<dyn SettingsReader>,
        policy: ExportUrlPolicy,
    ) -> Self {
        Self {
            reader,
            writer,
            tester,
            settings,
            policy,
        }
    }

    #[tracing::instrument(level = "info", skip(self))]
    pub async fn get(&self, org: Id<Organization>) -> Result<ExportTargetView, ServiceError> {
        let stored = self
            .reader
            .find(org)
            .await?
            .ok_or(RepositoryError::NotFound)?;
        Ok(ExportTargetView::from(&stored))
    }

    #[tracing::instrument(level = "info", skip(self, draft))]
    pub async fn put(
        &self,
        org: Id<Organization>,
        draft: ExportTargetDraft,
    ) -> Result<ExportTargetView, ServiceError> {
        // Deleting stays allowed: it hands the subtree back to the lock's export.
        if let Some(enforcer) = self.settings.resolution(org).await?.effective.enforced_by {
            return Err(ServiceError::SettingsEnforcedByAncestor {
                organization_id: enforcer.value(),
            });
        }
        self.policy.check(BASE_URL_FIELD, &draft.base_url)?;
        if let Some(token_url) = draft.auth.token_url() {
            self.policy.check(TOKEN_URL_FIELD, token_url)?;
        }
        let stored = self.reader.find(org).await?;
        let auth = draft
            .auth
            .resolve(&draft.base_url, stored.as_ref().map(|s| &s.target.config))?;
        let saved = self
            .writer
            .upsert(
                org,
                &ExportTargetConfig {
                    enabled: draft.enabled,
                    base_url: draft.base_url,
                    auth,
                },
            )
            .await?;
        Ok(ExportTargetView::from(&saved))
    }

    #[tracing::instrument(level = "info", skip(self))]
    pub async fn delete(&self, org: Id<Organization>) -> Result<(), ServiceError> {
        Ok(self.writer.delete(org).await?)
    }

    #[tracing::instrument(level = "info", skip(self))]
    pub async fn test(&self, org: Id<Organization>) -> Result<ConnectionReport, ServiceError> {
        let stored = self
            .reader
            .find(org)
            .await?
            .ok_or(RepositoryError::NotFound)?;
        Ok(self.tester.test(&stored.target.config).await)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Mutex};

    use chrono::Utc;
    use domain::data_export::{
        BASE_URL_FIELD, ConnectionOutcome, DataExportError, ExportAuthDraft, ExportPrincipal,
        ExportSecret, ExportTarget, ExportTargetStatus, ExportUrl, StoredExportTarget,
    };
    use secrecy::SecretString;
    use uuid::Uuid;

    use super::*;

    #[derive(Default)]
    struct Memory(Mutex<HashMap<Id<Organization>, StoredExportTarget>>);

    #[async_trait::async_trait]
    impl ExportTargetReader for Memory {
        async fn find(
            &self,
            org: Id<Organization>,
        ) -> Result<Option<StoredExportTarget>, RepositoryError> {
            Ok(self.0.lock().unwrap().get(&org).cloned())
        }
    }

    #[async_trait::async_trait]
    impl ExportTargetWriter for Memory {
        async fn upsert(
            &self,
            org: Id<Organization>,
            config: &ExportTargetConfig,
        ) -> Result<StoredExportTarget, RepositoryError> {
            let stored = StoredExportTarget {
                target: ExportTarget {
                    organization_id: org,
                    config: config.clone(),
                    activated_at: Utc::now(),
                },
                status: ExportTargetStatus::default(),
            };
            self.0.lock().unwrap().insert(org, stored.clone());
            Ok(stored)
        }

        async fn delete(&self, org: Id<Organization>) -> Result<(), RepositoryError> {
            self.0
                .lock()
                .unwrap()
                .remove(&org)
                .map(|_| ())
                .ok_or(RepositoryError::NotFound)
        }
    }

    struct FixedTester;

    #[async_trait::async_trait]
    impl ConnectionTester for FixedTester {
        async fn test(&self, _: &ExportTargetConfig) -> ConnectionReport {
            ConnectionReport::failed(ConnectionOutcome::Unreachable, None, None)
        }
    }

    /// `Some(enforcer)` locks every organization's settings from that ancestor.
    struct Lock(Option<Id<Organization>>);

    #[async_trait::async_trait]
    impl SettingsReader for Lock {
        async fn own(
            &self,
            _: Id<Organization>,
        ) -> Result<domain::settings::OrganizationSettings, RepositoryError> {
            unimplemented!()
        }
        async fn resolution(
            &self,
            o: Id<Organization>,
        ) -> Result<domain::settings::Resolution, RepositoryError> {
            use domain::settings::{
                DefectStreak, InstanceDefaults, MapView, OrganizationSettings, SensorOfflineAfter,
                resolve,
            };
            let defaults = InstanceDefaults {
                sensor_offline_after: SensorOfflineAfter::new(86_400).unwrap(),
                defect_streak: DefectStreak::new(3).unwrap(),
                map_view: MapView::new(
                    domain::shared::coordinates::Coordinate::new(54.79, 9.43).unwrap(),
                    None,
                )
                .unwrap(),
            };
            let mut chain: Vec<OrganizationSettings> = self
                .0
                .iter()
                .map(|enforcer| OrganizationSettings {
                    descendants_may_override: false,
                    ..OrganizationSettings::empty(*enforcer)
                })
                .collect();
            chain.push(OrganizationSettings::empty(o));
            Ok(resolve(&defaults, &chain, o))
        }
        async fn last_changes(
            &self,
            _: Id<Organization>,
        ) -> Result<
            HashMap<domain::settings::SettingKey, domain::settings::SettingChangeEntry>,
            RepositoryError,
        > {
            unimplemented!()
        }
    }

    fn service() -> DataExportService {
        service_locked_by(None)
    }

    fn service_locked_by(enforcer: Option<Id<Organization>>) -> DataExportService {
        let memory = Arc::new(Memory::default());
        DataExportService::new(
            memory.clone(),
            memory,
            Arc::new(FixedTester),
            Arc::new(Lock(enforcer)),
            ExportUrlPolicy::new(["example.org".to_string()], false),
        )
    }

    fn draft(url: &str, password: Option<&str>) -> ExportTargetDraft {
        ExportTargetDraft {
            enabled: true,
            base_url: ExportUrl::new(BASE_URL_FIELD, url).unwrap(),
            auth: ExportAuthDraft::Basic {
                username: ExportPrincipal::new("ge").unwrap(),
                password: password
                    .map(|p| ExportSecret::new("f", SecretString::from(p.to_string())).unwrap()),
            },
        }
    }

    fn org() -> Id<Organization> {
        Id::new(Uuid::now_v7())
    }

    #[tokio::test]
    async fn get_without_target_is_not_found() {
        assert!(matches!(
            service().get(org()).await,
            Err(ServiceError::Repository(RepositoryError::NotFound))
        ));
    }

    #[tokio::test]
    async fn put_refuses_a_target_below_a_settings_lock() {
        let enforcer = org();
        let err = service_locked_by(Some(enforcer))
            .put(org(), draft("https://frost.example.org/v1.1", Some("pw")))
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            ServiceError::SettingsEnforcedByAncestor { organization_id } if organization_id == enforcer.value()
        ));
    }

    #[tokio::test]
    async fn put_refuses_a_host_off_the_list() {
        let err = service()
            .put(org(), draft("https://evil.test/v1.1", Some("pw")))
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            ServiceError::DataExport(DataExportError::UrlNotAllowed {
                field: BASE_URL_FIELD
            })
        ));
    }

    #[tokio::test]
    async fn put_keeps_the_stored_secret_when_none_is_sent() {
        let service = service();
        let org = org();
        service
            .put(org, draft("https://frost.example.org/v1.1", Some("pw")))
            .await
            .unwrap();

        let view = service
            .put(org, draft("https://frost.example.org/v1.1", None))
            .await
            .unwrap();

        assert!(view.auth.has_secret);
    }

    #[tokio::test]
    async fn put_refuses_a_token_url_off_the_list() {
        let draft = ExportTargetDraft {
            auth: ExportAuthDraft::OAuth2ClientCredentials {
                token_url: ExportUrl::new(TOKEN_URL_FIELD, "https://auth.evil.test/token").unwrap(),
                client_id: ExportPrincipal::new("ge").unwrap(),
                client_secret: Some(
                    ExportSecret::new("f", SecretString::from("s".to_string())).unwrap(),
                ),
                scope: None,
            },
            ..draft("https://frost.example.org/v1.1", None)
        };

        let err = service().put(org(), draft).await.unwrap_err();

        assert!(matches!(
            err,
            ServiceError::DataExport(DataExportError::UrlNotAllowed {
                field: TOKEN_URL_FIELD
            })
        ));
    }

    #[tokio::test]
    async fn put_does_not_send_the_stored_secret_to_a_new_host() {
        let service = service();
        let org = org();
        service
            .put(org, draft("https://frost.example.org/v1.1", Some("pw")))
            .await
            .unwrap();

        let err = service
            .put(org, draft("https://other.example.org/v1.1", None))
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            ServiceError::DataExport(DataExportError::SecretMissing { kind: "basic" })
        ));
    }

    #[tokio::test]
    async fn the_first_put_without_secret_is_incomplete() {
        let err = service()
            .put(org(), draft("https://frost.example.org/v1.1", None))
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            ServiceError::DataExport(DataExportError::SecretMissing { .. })
        ));
    }

    #[tokio::test]
    async fn test_reports_what_the_tester_found() {
        let service = service();
        let org = org();
        service
            .put(org, draft("https://frost.example.org/v1.1", Some("pw")))
            .await
            .unwrap();

        let report = service.test(org).await.unwrap();

        assert_eq!(report.outcome, ConnectionOutcome::Unreachable);
    }

    #[tokio::test]
    async fn delete_of_a_missing_target_is_not_found() {
        assert!(matches!(
            service().delete(org()).await,
            Err(ServiceError::Repository(RepositoryError::NotFound))
        ));
    }
}
