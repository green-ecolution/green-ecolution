//! Service availability status.

use std::time::Duration;

use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceName {
    Postgres,
    Keycloak,
    Mqtt,
}

impl ServiceName {
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Postgres => "database",
            Self::Keycloak => "auth",
            Self::Mqtt => "mqtt",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceMessage {
    Connected,
    NoConnection,
    Disabled,
    ConnectionError,
    NotConfigured,
}

impl ServiceMessage {
    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Connected => "service.status.connected",
            Self::NoConnection => "service.status.no_connection",
            Self::Disabled => "service.status.disabled",
            Self::ConnectionError => "service.status.connection_error",
            Self::NotConfigured => "service.status.not_configured",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ServiceStatus {
    pub name: ServiceName,
    pub enabled: bool,
    pub healthy: bool,
    pub response_time: Duration,
    pub last_checked: DateTime<Utc>,
    pub message: ServiceMessage,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_name_keys_match_frontend_map() {
        assert_eq!(ServiceName::Postgres.as_key(), "database");
        assert_eq!(ServiceName::Keycloak.as_key(), "auth");
        assert_eq!(ServiceName::Mqtt.as_key(), "mqtt");
    }

    #[test]
    fn service_message_keys_match_frontend_i18n() {
        assert_eq!(ServiceMessage::Connected.as_key(), "service.status.connected");
        assert_eq!(ServiceMessage::NoConnection.as_key(), "service.status.no_connection");
        assert_eq!(ServiceMessage::Disabled.as_key(), "service.status.disabled");
        assert_eq!(ServiceMessage::ConnectionError.as_key(), "service.status.connection_error");
        assert_eq!(ServiceMessage::NotConfigured.as_key(), "service.status.not_configured");
    }
}
