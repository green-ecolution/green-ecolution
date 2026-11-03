use chrono::{DateTime, Utc};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain::data_export::{
    BASE_URL_FIELD, ConnectionOutcome, ConnectionReport, ExportAuthDraft, ExportAuthKind,
    ExportPrincipal, ExportRunStats, ExportScope, ExportSecret, ExportTargetDraft,
    ExportTargetView, ExportUrl, TOKEN_URL_FIELD,
};

use crate::service::ServiceError;

/// No `Debug`: the request carries the plaintext secret.
#[derive(Deserialize, utoipa::ToSchema)]
pub struct DataExportTargetRequest {
    pub enabled: bool,
    /// SensorThings service root including the version, e.g. `https://…/FROST-Server/v1.1`.
    pub base_url: String,
    pub auth: DataExportAuthRequest,
}

/// An omitted `password` / `client_secret` keeps the stored one of the same kind.
#[derive(Deserialize, utoipa::ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DataExportAuthRequest {
    None,
    Basic {
        username: String,
        #[serde(default)]
        #[schema(value_type = Option<String>, write_only)]
        password: Option<SecretString>,
    },
    #[serde(rename = "oauth2_client_credentials")]
    OAuth2ClientCredentials {
        token_url: String,
        client_id: String,
        #[serde(default)]
        #[schema(value_type = Option<String>, write_only)]
        client_secret: Option<SecretString>,
        #[serde(default)]
        scope: Option<String>,
    },
}

impl TryFrom<DataExportTargetRequest> for ExportTargetDraft {
    type Error = ServiceError;

    fn try_from(req: DataExportTargetRequest) -> Result<Self, Self::Error> {
        let auth = match req.auth {
            DataExportAuthRequest::None => ExportAuthDraft::None,
            DataExportAuthRequest::Basic { username, password } => ExportAuthDraft::Basic {
                username: ExportPrincipal::new(username)?,
                password: password
                    .map(|p| ExportSecret::new("data_export.auth.password", p))
                    .transpose()?,
            },
            DataExportAuthRequest::OAuth2ClientCredentials {
                token_url,
                client_id,
                client_secret,
                scope,
            } => ExportAuthDraft::OAuth2ClientCredentials {
                token_url: ExportUrl::new(TOKEN_URL_FIELD, &token_url)?,
                client_id: ExportPrincipal::new(client_id)?,
                client_secret: client_secret
                    .map(|s| ExportSecret::new("data_export.auth.client_secret", s))
                    .transpose()?,
                scope: scope.map(ExportScope::new).transpose()?,
            },
        };
        Ok(Self {
            enabled: req.enabled,
            base_url: ExportUrl::new(BASE_URL_FIELD, &req.base_url)?,
            auth,
        })
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DataExportAuthKindDto {
    None,
    Basic,
    #[serde(rename = "oauth2_client_credentials")]
    OAuth2ClientCredentials,
}

impl From<ExportAuthKind> for DataExportAuthKindDto {
    fn from(kind: ExportAuthKind) -> Self {
        match kind {
            ExportAuthKind::None => Self::None,
            ExportAuthKind::Basic => Self::Basic,
            ExportAuthKind::OAuth2ClientCredentials => Self::OAuth2ClientCredentials,
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DataExportAuthResponse {
    pub kind: DataExportAuthKindDto,
    pub username: Option<String>,
    pub client_id: Option<String>,
    pub token_url: Option<String>,
    pub scope: Option<String>,
    /// The secret itself is never returned.
    pub has_secret: bool,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DataExportRunStatsResponse {
    pub things_written: u32,
    pub observations_written: u32,
    pub skipped: u32,
}

impl From<ExportRunStats> for DataExportRunStatsResponse {
    fn from(s: ExportRunStats) -> Self {
        Self {
            things_written: s.things_written,
            observations_written: s.observations_written,
            skipped: s.skipped,
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DataExportStatusResponse {
    pub last_run_at: Option<DateTime<Utc>>,
    pub last_success_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub last_run_stats: Option<DataExportRunStatsResponse>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DataExportTargetResponse {
    pub organization_id: Uuid,
    /// Always `sensorthings` for now.
    pub kind: String,
    pub enabled: bool,
    pub base_url: String,
    pub auth: DataExportAuthResponse,
    pub activated_at: DateTime<Utc>,
    pub status: DataExportStatusResponse,
}

impl From<ExportTargetView> for DataExportTargetResponse {
    fn from(view: ExportTargetView) -> Self {
        let (username, client_id) = match view.auth.kind {
            ExportAuthKind::Basic => (view.auth.principal, None),
            ExportAuthKind::OAuth2ClientCredentials => (None, view.auth.principal),
            ExportAuthKind::None => (None, None),
        };
        Self {
            organization_id: view.organization_id.value(),
            kind: "sensorthings".to_string(),
            enabled: view.enabled,
            base_url: view.base_url,
            auth: DataExportAuthResponse {
                kind: view.auth.kind.into(),
                username,
                client_id,
                token_url: view.auth.token_url,
                scope: view.auth.scope,
                has_secret: view.auth.has_secret,
            },
            activated_at: view.activated_at,
            status: DataExportStatusResponse {
                last_run_at: view.status.last_run_at,
                last_success_at: view.status.last_success_at,
                last_error: view.status.last_error,
                last_run_stats: view.status.last_run_stats.map(Into::into),
            },
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionOutcomeDto {
    Ok,
    UrlNotAllowed,
    Unreachable,
    Unauthorized,
    NotSensorthings,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ConnectionTestResponse {
    pub outcome: ConnectionOutcomeDto,
    pub http_status: Option<u16>,
    /// The server lists the SensorThings data-array conformance class, which the export uses for readings.
    pub data_array_supported: bool,
    pub detail: Option<String>,
}

impl From<ConnectionReport> for ConnectionTestResponse {
    fn from(r: ConnectionReport) -> Self {
        Self {
            outcome: match r.outcome {
                ConnectionOutcome::Ok => ConnectionOutcomeDto::Ok,
                ConnectionOutcome::UrlNotAllowed => ConnectionOutcomeDto::UrlNotAllowed,
                ConnectionOutcome::Unreachable => ConnectionOutcomeDto::Unreachable,
                ConnectionOutcome::Unauthorized => ConnectionOutcomeDto::Unauthorized,
                ConnectionOutcome::NotSensorThings => ConnectionOutcomeDto::NotSensorthings,
            },
            http_status: r.http_status,
            data_array_supported: r.data_array_supported,
            detail: r.detail,
        }
    }
}
