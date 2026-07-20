use std::{collections::HashMap, str::FromStr};

use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use domain::{
    RepositoryError,
    shared::email::Email,
    user::{UserIdentity, UserRole, Username},
};

const ATTR_USER_ROLES: &str = "user_roles";

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

        let attributes = self.attributes.unwrap_or_default();
        let roles = parse_attr_list::<UserRole, _>(&attributes, ATTR_USER_ROLES)?;

        Ok(UserIdentity {
            id,
            created_at,
            username: Username::reconstitute(self.username.unwrap_or_default()),
            first_name: self.first_name.unwrap_or_default(),
            last_name: self.last_name.unwrap_or_default(),
            email: Email::reconstitute(self.email.unwrap_or_default()),
            email_verified: self.email_verified.unwrap_or(false),
            roles,
        })
    }
}

fn parse_attr_list<T, E>(
    attrs: &HashMap<String, Vec<String>>,
    key: &str,
) -> Result<Vec<T>, RepositoryError>
where
    T: FromStr<Err = E>,
    E: Into<RepositoryError>,
{
    let Some(values) = attrs.get(key) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for value in values {
        for piece in value.split(',') {
            let trimmed = piece.trim();
            if trimmed.is_empty() {
                continue;
            }
            out.push(T::from_str(trimmed).map_err(|e| e.into())?);
        }
    }
    Ok(out)
}

#[derive(Debug, Serialize)]
pub struct KcCredential<'a> {
    #[serde(rename = "type")]
    pub kind: &'a str,
    pub value: &'a str,
    pub temporary: bool,
}

#[derive(Debug, Serialize)]
pub struct KcRoleRepresentation<'a> {
    pub id: &'a str,
    pub name: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kc_user_with_attrs(attrs: HashMap<String, Vec<String>>) -> KcUser {
        KcUser {
            id: Some("11111111-2222-3333-4444-555555555555".into()),
            created_timestamp: Some(1_700_000_000_000),
            username: Some("jdoe".into()),
            first_name: Some("John".into()),
            last_name: Some("Doe".into()),
            email: Some("j@d.de".into()),
            email_verified: Some(true),
            enabled: Some(true),
            attributes: Some(attrs),
        }
    }

    #[test]
    fn maps_user_attributes() {
        let mut attrs = HashMap::new();
        attrs.insert("user_roles".into(), vec!["tbz,green-ecolution".into()]);

        let user = kc_user_with_attrs(attrs).try_into_identity().unwrap();
        assert_eq!(user.username.as_str(), "jdoe");
        assert_eq!(user.roles, vec![UserRole::Tbz, UserRole::GreenEcolution]);
    }

    #[test]
    fn missing_attributes_are_optional() {
        let user = kc_user_with_attrs(HashMap::new())
            .try_into_identity()
            .unwrap();
        assert!(user.roles.is_empty());
    }
}
