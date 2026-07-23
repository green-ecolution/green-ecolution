use std::collections::HashMap;

use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain::{
    RepositoryError,
    shared::email::Email,
    user::{UserIdentity, Username},
};

// Custom user metadata lives in `attributes` as `Vec<String>` per key (Keycloak quirk).
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct KcUser {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default, rename = "createdTimestamp")]
    pub created_timestamp: Option<i64>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default, rename = "firstName")]
    pub first_name: Option<String>,
    #[serde(default, rename = "lastName")]
    pub last_name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default, rename = "emailVerified")]
    pub email_verified: Option<bool>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub attributes: Option<HashMap<String, Vec<String>>>,
}

impl KcUser {
    pub fn try_into_identity(self) -> Result<UserIdentity, RepositoryError> {
        let id_str = self
            .id
            .ok_or_else(|| RepositoryError::DataIntegrity("keycloak user missing id".into()))?;
        let id = Uuid::parse_str(&id_str).map_err(|e| {
            RepositoryError::DataIntegrity(format!("keycloak user id is not a uuid: {e}"))
        })?;

        let created_at = self
            .created_timestamp
            .and_then(|ts| Utc.timestamp_millis_opt(ts).single())
            .unwrap_or_else(Utc::now);

        Ok(UserIdentity {
            id,
            created_at,
            username: Username::reconstitute(self.username.unwrap_or_default()),
            first_name: self.first_name.unwrap_or_default(),
            last_name: self.last_name.unwrap_or_default(),
            email: Email::reconstitute(self.email.unwrap_or_default()),
            email_verified: self.email_verified.unwrap_or(false),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct KcCredential<'a> {
    #[serde(rename = "type")]
    pub kind: &'a str,
    pub value: &'a str,
    pub temporary: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_identity_fields_without_roles() {
        let kc = KcUser {
            id: Some("11111111-2222-3333-4444-555555555555".into()),
            created_timestamp: Some(1_700_000_000_000),
            username: Some("jdoe".into()),
            first_name: Some("John".into()),
            last_name: Some("Doe".into()),
            email: Some("j@d.de".into()),
            email_verified: Some(true),
            enabled: Some(true),
            attributes: None,
        };

        let user = kc.try_into_identity().unwrap();

        assert_eq!(
            user.id,
            Uuid::parse_str("11111111-2222-3333-4444-555555555555").unwrap()
        );
        assert_eq!(user.username.as_str(), "jdoe");
        assert_eq!(user.email.as_str(), "j@d.de");
        assert!(user.email_verified);
    }
}
