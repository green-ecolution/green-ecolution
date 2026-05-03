use std::{collections::HashMap, str::FromStr};

use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::domain::{
    RepositoryError,
    shared::email::Email,
    user::{User, UserRole, UserStatus, Username},
    vehicle::DrivingLicense,
};

const ATTR_PHONE_NUMBER: &str = "phone_number";
const ATTR_EMPLOYEE_ID: &str = "employee_id";
const ATTR_AVATAR_URL: &str = "avatar_url";
const ATTR_USER_ROLES: &str = "user_roles";
const ATTR_DRIVING_LICENSES: &str = "driving_licenses";
const ATTR_STATUS: &str = "status";

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
    pub fn try_into_domain(self) -> Result<User, RepositoryError> {
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
        let phone_number = first_attr(&attributes, ATTR_PHONE_NUMBER);
        let employee_id = first_attr(&attributes, ATTR_EMPLOYEE_ID);
        let avatar_url = first_attr(&attributes, ATTR_AVATAR_URL)
            .as_deref()
            .map(Url::parse)
            .transpose()
            .map_err(|e| RepositoryError::DataIntegrity(format!("invalid avatar_url: {e}")))?;

        let roles = parse_attr_list::<UserRole, _>(&attributes, ATTR_USER_ROLES)?;
        let driving_licenses =
            parse_attr_list::<DrivingLicense, _>(&attributes, ATTR_DRIVING_LICENSES)?;
        let status = first_attr(&attributes, ATTR_STATUS)
            .as_deref()
            .map(UserStatus::from_str)
            .transpose()
            .map_err(RepositoryError::from)?
            .unwrap_or(UserStatus::Available);

        Ok(User {
            id,
            created_at,
            username: Username::reconstitute(self.username.unwrap_or_default()),
            first_name: self.first_name.unwrap_or_default(),
            last_name: self.last_name.unwrap_or_default(),
            email: Email::reconstitute(self.email.unwrap_or_default()),
            email_verified: self.email_verified.unwrap_or(false),
            employee_id,
            phone_number,
            avatar_url,
            roles,
            driving_licenses,
            status,
        })
    }
}

fn first_attr(attrs: &HashMap<String, Vec<String>>, key: &str) -> Option<String> {
    attrs
        .get(key)
        .and_then(|values| values.first())
        .filter(|s| !s.is_empty())
        .cloned()
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
        attrs.insert("phone_number".into(), vec!["+49 123".into()]);
        attrs.insert("employee_id".into(), vec!["EMP-1".into()]);
        attrs.insert("user_roles".into(), vec!["tbz,green-ecolution".into()]);
        attrs.insert("driving_licenses".into(), vec!["B,CE".into()]);
        attrs.insert("status".into(), vec!["available".into()]);

        let user = kc_user_with_attrs(attrs).try_into_domain().unwrap();
        assert_eq!(user.username.as_str(), "jdoe");
        assert_eq!(user.phone_number.as_deref(), Some("+49 123"));
        assert_eq!(user.employee_id.as_deref(), Some("EMP-1"));
        assert_eq!(user.roles, vec![UserRole::Tbz, UserRole::GreenEcolution]);
        assert_eq!(
            user.driving_licenses,
            vec![DrivingLicense::B, DrivingLicense::CE]
        );
        assert_eq!(user.status, UserStatus::Available);
    }

    #[test]
    fn missing_attributes_are_optional() {
        let user = kc_user_with_attrs(HashMap::new())
            .try_into_domain()
            .unwrap();
        assert!(user.roles.is_empty());
        assert_eq!(user.status, UserStatus::Available);
    }
}
