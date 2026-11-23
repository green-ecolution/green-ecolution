use std::collections::BTreeMap;

use domain::module::{ModuleKey, ModuleState};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ModuleListResponse {
    /// Modules released in at least one organization the caller works in.
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
    /// The organization switches this module itself. Sub-organizations
    /// follow their tenant and report `false`.
    pub switchable: bool,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct OrganizationModulesResponse {
    pub modules: Vec<OrganizationModuleResponse>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct OrganizationModulesUpdateRequest {
    /// `true` switches a released module on, `false` off; an absent key
    /// stays as it is.
    #[serde(default)]
    pub modules: BTreeMap<String, bool>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ModuleReleaseResponse {
    pub key: String,
    pub released: bool,
    /// The tenant's own switch; kept while the module is locked.
    pub enabled: bool,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TenantModuleReleasesResponse {
    pub organization_id: Uuid,
    pub modules: Vec<ModuleReleaseResponse>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ModuleReleasesResponse {
    /// Every module this instance ships.
    pub modules: Vec<String>,
    pub tenants: Vec<TenantModuleReleasesResponse>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ModuleReleasesUpdateRequest {
    /// `true` releases a module for the tenant, `false` locks it; an absent
    /// key stays as it is.
    #[serde(default)]
    pub modules: BTreeMap<String, bool>,
}

impl TenantModuleReleasesResponse {
    pub fn build(tenant: Uuid, states: &BTreeMap<ModuleKey, ModuleState>) -> Self {
        Self {
            organization_id: tenant,
            modules: states
                .iter()
                .map(|(key, state)| ModuleReleaseResponse {
                    key: key.to_string(),
                    released: state.released,
                    enabled: state.enabled,
                })
                .collect(),
        }
    }
}

impl OrganizationModulesResponse {
    /// Lists released modules only; a locked one is absent.
    pub fn build(states: &BTreeMap<ModuleKey, ModuleState>, switchable: bool) -> Self {
        let modules = states
            .iter()
            .filter(|(_, state)| state.released)
            .map(|(key, state)| OrganizationModuleResponse {
                key: key.to_string(),
                enabled: state.enabled,
                switchable,
            })
            .collect();
        Self { modules }
    }
}
