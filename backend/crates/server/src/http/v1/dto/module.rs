use std::collections::BTreeMap;

use domain::{
    module::{ActivationResolution, ModuleKey},
    settings::SettingOrigin,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::http::v1::dto::settings::{OrganizationRef, SettingOriginDto};

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ModuleListResponse {
    /// Every module this instance ships, active or not.
    pub modules: Vec<String>,
    /// One entry per organization the caller works in.
    pub organizations: Vec<OrganizationModulesEntry>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct OrganizationModulesEntry {
    pub organization_id: Uuid,
    pub active: Vec<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct OrganizationModuleResponse {
    pub key: String,
    pub enabled: bool,
    pub origin: SettingOriginDto,
    /// Set only where `origin` is `inherited`.
    pub source: Option<OrganizationRef>,
    /// This organization's own switch, reported even while a lock above makes it dormant.
    pub own_value: Option<bool>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct OrganizationModulesResponse {
    pub modules: Vec<OrganizationModuleResponse>,
    pub enforced_by: Option<OrganizationRef>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct OrganizationModulesUpdateRequest {
    /// `true`/`false` sets the organization's own switch, `null` returns the
    /// module to inheritance, an absent key stays as it is.
    #[serde(default)]
    pub modules: BTreeMap<String, Option<bool>>,
}

impl OrganizationModulesResponse {
    pub fn build(
        resolution: &BTreeMap<ModuleKey, ActivationResolution>,
        own: &BTreeMap<ModuleKey, bool>,
    ) -> Self {
        let enforced_by = resolution
            .values()
            .find_map(|r| r.enforced_by)
            .map(|id| OrganizationRef { id: id.value() });
        let modules = resolution
            .iter()
            .map(|(key, r)| OrganizationModuleResponse {
                key: key.to_string(),
                enabled: r.enabled,
                origin: match r.origin {
                    SettingOrigin::Default => SettingOriginDto::Default,
                    SettingOrigin::Inherited(_) => SettingOriginDto::Inherited,
                    SettingOrigin::Own => SettingOriginDto::Own,
                },
                source: match r.origin {
                    SettingOrigin::Inherited(id) => Some(OrganizationRef { id: id.value() }),
                    _ => None,
                },
                own_value: own.get(key).copied(),
            })
            .collect();
        Self {
            modules,
            enforced_by,
        }
    }
}
