use url::Url;
use uuid::Uuid;

use crate::shared::phone_number::PhoneNumber;
use crate::vehicle::DrivingLicense;

use super::UserStatus;

/// App-owned user facts, merged with the IdP identity into `UserView`.
///
/// Deliberately not an aggregate: there are no invariants between the fields
/// and no events to emit. The struct doubles as the replace-style write
/// input since it carries its own id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserProfile {
    pub id: Uuid,
    pub employee_id: Option<String>,
    pub phone_number: Option<PhoneNumber>,
    pub avatar_url: Option<Url>,
    pub status: UserStatus,
    pub driving_licenses: Vec<DrivingLicense>,
    /// Defaults to true: a person without maintained profile facts must not
    /// silently drop out of watering-plan assignment.
    pub watering_plan_selectable: bool,
}

impl UserProfile {
    pub fn empty(id: Uuid) -> Self {
        Self {
            id,
            employee_id: None,
            phone_number: None,
            avatar_url: None,
            status: UserStatus::Available,
            driving_licenses: Vec::new(),
            watering_plan_selectable: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_profile_has_defaults() {
        let id = Uuid::now_v7();
        let p = UserProfile::empty(id);
        assert_eq!(p.id, id);
        assert_eq!(p.status, UserStatus::Available);
        assert!(p.driving_licenses.is_empty());
        assert!(p.employee_id.is_none());
        assert!(p.phone_number.is_none());
        assert!(p.avatar_url.is_none());
        assert!(p.watering_plan_selectable);
    }
}
