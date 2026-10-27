pub mod authorization;
pub mod comment_service;
pub mod event_bus;
pub mod module_access;
pub mod module_activation_service;
pub mod operation_execution_service;
pub mod operation_plan_service;
pub mod organization_service;
pub mod planning;
pub mod plugin_proxy_policy;
pub mod plugin_proxy_service;
pub mod plugin_sensor_ingest_service;
pub mod plugin_service;
pub mod plugin_view_ticket_service;
pub mod region_service;
pub mod role_service;
pub mod sensor_parsers;
pub mod sensor_service;
pub mod settings_service;
pub mod start_point_service;
pub mod statistics_registry;
pub mod targets;
pub mod user_service;
pub mod vehicle_service;

/// A stand-in module resource: the core's unit tests run without any module crate.
#[cfg(test)]
pub(crate) mod test_resources {
    use domain::authorization::{ResourceCatalog, ResourceKey};

    pub const WIDGET: ResourceKey = ResourceKey::declare("widget");

    pub fn catalog() -> ResourceCatalog {
        ResourceCatalog::new([WIDGET]).unwrap()
    }
}

use domain::{
    RepositoryError,
    operation_plan::OperationPlanError,
    routing::RoutingError,
    shared::error::{ValidationError, ValidationIssue},
};

/// Optional subsystem a request can hit while it is switched off.
///
/// A closed set rather than a `&'static str` so every feature gets its own
/// stable error code instead of one opaque "something is disabled".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feature {
    Routing,
    Plugins,
}

impl std::fmt::Display for Feature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Routing => "routing",
            Self::Plugins => "plugins",
        })
    }
}

/// Which request part failed to parse.
///
/// Each gets its own error code because a client wants to point at a different
/// input for each: a date picker, an entity selection, a scanned sensor id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Malformed {
    Date,
    Uuid,
    SensorId,
    BoundingBox,
    Permission,
    PluginFrontend,
}

impl Malformed {
    fn code(self) -> &'static str {
        match self {
            Self::Date => "request.malformed_date",
            Self::Uuid => "request.malformed_uuid",
            Self::SensorId => "request.malformed_sensor_id",
            Self::BoundingBox => "request.malformed_bounding_box",
            Self::Permission => "request.unknown_permission",
            Self::PluginFrontend => "request.malformed_plugin_frontend",
        }
    }
}

impl std::fmt::Display for Malformed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Date => "date",
            Self::Uuid => "id",
            Self::SensorId => "sensor id",
            Self::BoundingBox => "bounding box",
            Self::Permission => "permission",
            Self::PluginFrontend => "plugin frontend",
        })
    }
}

/// Maps to 409 (`Conflict`) or 422 (`Unprocessable`); the stable error code
/// stays module-defined.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleErrorKind {
    Conflict,
    Unprocessable,
}

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    /// A single input field violated a domain rule. Carries the typed error so
    /// the field label, the violated rule and its parameters survive to the
    /// client instead of collapsing into a sentence.
    #[error(transparent)]
    Validation(#[from] ValidationError),
    /// A request part the server could not even parse into a domain value.
    /// Distinct from `Validation`, which means a value parsed but broke a rule.
    #[error("malformed {kind}: {detail}")]
    Malformed { kind: Malformed, detail: String },
    /// A rule the request broke that is not attributable to one input field,
    /// e.g. two request parts that contradict each other.
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error("target already has a different sensor")]
    TargetAlreadyHasSensor,
    #[error("sensor is already attached to another target")]
    SensorAlreadyAttached,
    #[error("sensor is already activated")]
    AlreadyActivated,
    #[error("sensor is not activated")]
    NotActivated,
    #[error("{feature} feature is disabled")]
    FeatureDisabled { feature: Feature },
    /// A plugin ingest batch exceeded the endpoint's item limit.
    #[error("batch exceeds the limit of {limit} items")]
    PayloadTooLarge { limit: usize },
    #[error(transparent)]
    Routing(#[from] RoutingError),
    #[error(transparent)]
    Organization(#[from] domain::organization::OrganizationError),
    #[error(transparent)]
    Role(#[from] domain::role::RoleError),
    #[error("organization still has sub-organizations or users")]
    OrganizationNotEmpty,
    #[error(transparent)]
    OrganizationMismatch(#[from] OrganizationMismatch),
    #[error("contact person is not a member of this organization")]
    ContactPersonNotAMember,
    #[error("sensor is attached to a target and must be transferred with it")]
    SensorBoundToTarget,
    #[error("no organization given and the acting user has none")]
    MissingOrganization,
    #[error("a user cannot change their own roles or organization")]
    CannotChangeOwnAccess,
    #[error("this change would remove your own right to administer roles and members")]
    CannotRevokeOwnAdministration,
    /// A write below an organization that froze its subtree. Not a missing
    /// permission but a state conflict, hence 409 rather than 403.
    #[error("settings are enforced by organization {organization_id}")]
    SettingsEnforcedByAncestor { organization_id: uuid::Uuid },
    #[error("the plugin proxy is not configured on this instance")]
    PluginProxyUnavailable,
    #[error("proxied frontend target is not on the service allowlist")]
    PluginProxyTargetNotAllowed,
    #[error("plugin upstream unreachable")]
    PluginUpstreamUnreachable,
    #[error("unknown module {key}")]
    ModuleUnknown { key: String },
    /// The module exists but is switched off for the organization the request
    /// targets. Not a missing permission of the user, but still a refusal.
    #[error("module {module} is not active in this organization")]
    ModuleDisabled { module: String },
    #[error("module {module} does not accept {kind} targets for this purpose")]
    TargetNotAccepted { module: String, kind: String },
    #[error("{message}")]
    TargetRefused { code: &'static str, message: String },
    #[error("vehicle {vehicle} has no {kind} capacity")]
    VehicleCapacityMissing { vehicle: uuid::Uuid, kind: String },
    #[error(transparent)]
    OperationPlan(#[from] OperationPlanError),
    #[error("unknown stop {stop}")]
    UnknownStop { stop: uuid::Uuid },
    /// An error a module raises with its own stable code, e.g.
    /// `conflict.tree_in_cluster`.
    #[error("{message}")]
    Module {
        kind: ModuleErrorKind,
        code: &'static str,
        message: String,
    },
}

impl ServiceError {
    /// Stable discriminator the client keys its own wording on.
    ///
    /// The `Display` text is prose and may be reworded; only this is contract.
    /// The match is exhaustive on purpose, so a new variant cannot ship without
    /// deciding what a client should be able to tell apart.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Repository(e) => repository_code(e),
            Self::Validation(_) => "request.validation_failed",
            Self::Malformed { kind, .. } => kind.code(),
            Self::InvalidInput(_) => "request.invalid_input",
            Self::Auth(e) => e.code(),
            Self::Routing(e) => routing_code(e),
            Self::Organization(e) => organization_code(e),
            Self::Role(e) => role_code(e),
            Self::OperationPlan(e) => operation_plan_code(e),
            Self::OrganizationMismatch(kind) => kind.code(),
            Self::TargetAlreadyHasSensor => "conflict.target_already_has_sensor",
            Self::SensorAlreadyAttached => "sensor.already_attached",
            Self::AlreadyActivated => "conflict.sensor_already_activated",
            Self::NotActivated => "conflict.sensor_not_activated",
            Self::OrganizationNotEmpty => "conflict.organization_not_empty",
            Self::SensorBoundToTarget => "conflict.sensor_bound_to_target",
            Self::CannotChangeOwnAccess => "conflict.cannot_change_own_access",
            Self::CannotRevokeOwnAdministration => "conflict.cannot_revoke_own_administration",
            Self::SettingsEnforcedByAncestor { .. } => "settings.enforced_by_ancestor",
            Self::ContactPersonNotAMember => "organization.contact_person_not_a_member",
            Self::MissingOrganization => "organization.missing",
            Self::ModuleUnknown { .. } => "module.unknown",
            Self::ModuleDisabled { .. } => "module.disabled",
            Self::TargetNotAccepted { .. } => "target.not_accepted",
            Self::VehicleCapacityMissing { .. } => "vehicle.capacity_missing",
            Self::UnknownStop { .. } => "plan.unknown_stop",
            Self::TargetRefused { code, .. } => code,
            Self::Module { code, .. } => code,
            Self::FeatureDisabled { feature } => match feature {
                Feature::Routing => "feature.routing_disabled",
                Feature::Plugins => "feature.plugins_disabled",
            },
            Self::PayloadTooLarge { .. } => "plugin.batch_too_large",
            Self::PluginProxyUnavailable => "plugin.proxy_unavailable",
            Self::PluginProxyTargetNotAllowed => "plugin.proxy_target_not_allowed",
            Self::PluginUpstreamUnreachable => "plugin.upstream_unreachable",
        }
    }

    /// Field-level detail, present only where one input field is to blame.
    ///
    /// Built through the shared [`ValidationIssue`] constructor, so a rule the
    /// browser already checks and the same rule enforced here resolve to the
    /// same translation key.
    pub fn validation_issue(&self) -> Option<ValidationIssue> {
        let error = match self {
            Self::Validation(e) => e,
            Self::Organization(domain::organization::OrganizationError::Validation(e)) => e,
            Self::Role(domain::role::RoleError::Validation(e)) => e,
            Self::OperationPlan(OperationPlanError::Validation(e)) => e,
            _ => return None,
        };
        Some(ValidationIssue::from(error))
    }
}

fn repository_code(e: &RepositoryError) -> &'static str {
    match e {
        RepositoryError::NotFound => "resource.not_found",
        RepositoryError::AlreadyExists(_) => "resource.already_exists",
        RepositoryError::ForeignKeyViolation(_) => "resource.referenced_missing",
        RepositoryError::ConstraintViolation(_) => "resource.constraint_violated",
        RepositoryError::DataIntegrity(_) | RepositoryError::Internal(_) => "internal.error",
    }
}

fn routing_code(e: &RoutingError) -> &'static str {
    match e {
        RoutingError::Unavailable(_) => "routing.unavailable",
        RoutingError::InvalidProblem(_) => "routing.invalid_problem",
        RoutingError::Failed(_) => "routing.failed",
        RoutingError::NoStation { .. } => "routing.no_station_for_capacity",
    }
}

fn organization_code(e: &domain::organization::OrganizationError) -> &'static str {
    use domain::organization::OrganizationError as E;
    match e {
        E::RootImmutable => "conflict.root_organization_immutable",
        E::Validation(_) => "request.invalid_input",
    }
}

fn role_code(e: &domain::role::RoleError) -> &'static str {
    use domain::role::RoleError as E;
    match e {
        E::TemplateImmutable => "conflict.role_template_immutable",
        E::CannotAssignTemplate => "conflict.role_template_not_assignable",
        E::Validation(_) => "request.invalid_input",
    }
}

fn operation_plan_code(e: &OperationPlanError) -> &'static str {
    use OperationPlanError as E;
    match e {
        E::InvalidStateTransition { .. } => "plan.invalid_state_transition",
        E::CannotMutateAfterStart => "plan.cannot_mutate_after_start",
        E::CancellationNoteRequired => "plan.cancellation_note_required",
        E::OutcomeMissing(_) => "plan.outcome_missing",
        E::OutcomeDuplicate(_) => "plan.outcome_duplicate",
        E::OutcomeUnknownStop(_) => "plan.outcome_unknown_stop",
        E::TransporterCapacityRequired => "plan.transporter_capacity_required",
        E::Validation(_) => "request.invalid_input",
    }
}

/// Which cross-aggregate organization rule a request violated.
///
/// Two entities may only be linked while they belong to the same
/// organization; a request that would cross that boundary is rejected here
/// rather than silently pulling one side along. The variants exist so clients
/// can tell the cases apart — the `code` is a stable wire contract, the
/// `Display` text is a fallback for clients without their own wording.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum OrganizationMismatch {
    #[error("the sensor belongs to a different organization than its target")]
    SensorVsTarget,
    #[error("the selected stops are outside the plan's organization")]
    StopsVsPlan,
    #[error("the role belongs to a different organization than the user")]
    RoleVsUser,
}

impl OrganizationMismatch {
    pub fn code(self) -> &'static str {
        match self {
            Self::SensorVsTarget => "organization_mismatch.sensor_vs_target",
            Self::StopsVsPlan => "organization_mismatch.stops_vs_plan",
            Self::RoleVsUser => "organization_mismatch.role_vs_user",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("missing or malformed authorization header")]
    MissingToken,
    #[error("invalid token: {0}")]
    InvalidToken(String),
    #[error("token expired")]
    TokenExpired,
    #[error("forbidden: missing required permission")]
    Forbidden,
    #[error("identity provider unavailable: {0}")]
    IdpUnavailable(String),
    /// Separate from `InvalidToken`: a plugin key is not a JWT, and the code
    /// must let a client tell it apart from a bad user token without parsing
    /// prose. Never carries the key, the secret or the hash.
    #[error("invalid plugin key")]
    PluginKeyInvalid,
    #[error("plugin is disabled")]
    PluginDisabled,
    #[error("invalid plugin view ticket")]
    PluginViewTicketInvalid,
    #[error("invalid plugin proxy session")]
    PluginProxySessionInvalid,
}

impl AuthError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::MissingToken => "auth.missing_token",
            Self::InvalidToken(_) => "auth.invalid_token",
            Self::TokenExpired => "auth.token_expired",
            Self::Forbidden => "auth.forbidden",
            Self::IdpUnavailable(_) => "auth.idp_unavailable",
            Self::PluginKeyInvalid => "plugin.key_invalid",
            Self::PluginDisabled => "plugin.disabled",
            Self::PluginViewTicketInvalid => "plugin.view_ticket_invalid",
            Self::PluginProxySessionInvalid => "plugin.proxy_session_invalid",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::operation_plan::OperationPlanStatus;
    use domain::planning::StopRef;
    use std::collections::HashSet;

    /// One instance per `ServiceError` variant. `code()` matches exhaustively,
    /// so a new variant breaks the build; this list keeps the wire contract
    /// itself under test.
    fn every_variant() -> Vec<ServiceError> {
        vec![
            ServiceError::Repository(RepositoryError::NotFound),
            ServiceError::Repository(RepositoryError::AlreadyExists("x".into())),
            ServiceError::Repository(RepositoryError::ForeignKeyViolation("x".into())),
            ServiceError::Repository(RepositoryError::ConstraintViolation("x".into())),
            ServiceError::Repository(RepositoryError::DataIntegrity("x".into())),
            ServiceError::Repository(RepositoryError::Internal("x".into())),
            ServiceError::Validation(ValidationError::EmptyString {
                field: "tree.species",
            }),
            ServiceError::Malformed {
                kind: Malformed::Date,
                detail: "x".into(),
            },
            ServiceError::Malformed {
                kind: Malformed::Uuid,
                detail: "x".into(),
            },
            ServiceError::Malformed {
                kind: Malformed::SensorId,
                detail: "x".into(),
            },
            ServiceError::Malformed {
                kind: Malformed::BoundingBox,
                detail: "x".into(),
            },
            ServiceError::Malformed {
                kind: Malformed::Permission,
                detail: "x".into(),
            },
            ServiceError::Malformed {
                kind: Malformed::PluginFrontend,
                detail: "x".into(),
            },
            ServiceError::InvalidInput("x".into()),
            ServiceError::Auth(AuthError::MissingToken),
            ServiceError::Auth(AuthError::InvalidToken("x".into())),
            ServiceError::Auth(AuthError::TokenExpired),
            ServiceError::Auth(AuthError::Forbidden),
            ServiceError::Auth(AuthError::IdpUnavailable("x".into())),
            ServiceError::Auth(AuthError::PluginKeyInvalid),
            ServiceError::Auth(AuthError::PluginDisabled),
            ServiceError::Auth(AuthError::PluginViewTicketInvalid),
            ServiceError::TargetAlreadyHasSensor,
            ServiceError::SensorAlreadyAttached,
            ServiceError::AlreadyActivated,
            ServiceError::NotActivated,
            ServiceError::FeatureDisabled {
                feature: Feature::Routing,
            },
            ServiceError::FeatureDisabled {
                feature: Feature::Plugins,
            },
            ServiceError::PayloadTooLarge { limit: 500 },
            ServiceError::Routing(RoutingError::Unavailable("x".into())),
            ServiceError::Routing(RoutingError::InvalidProblem("x".into())),
            ServiceError::Routing(RoutingError::Failed("x".into())),
            ServiceError::Routing(RoutingError::NoStation {
                kind: domain::capacity::CapacityKind::CargoVolume,
            }),
            ServiceError::Organization(domain::organization::OrganizationError::RootImmutable),
            ServiceError::Role(domain::role::RoleError::TemplateImmutable),
            ServiceError::Role(domain::role::RoleError::CannotAssignTemplate),
            ServiceError::OrganizationNotEmpty,
            ServiceError::OrganizationMismatch(OrganizationMismatch::SensorVsTarget),
            ServiceError::OrganizationMismatch(OrganizationMismatch::StopsVsPlan),
            ServiceError::OrganizationMismatch(OrganizationMismatch::RoleVsUser),
            ServiceError::ContactPersonNotAMember,
            ServiceError::SensorBoundToTarget,
            ServiceError::MissingOrganization,
            ServiceError::CannotChangeOwnAccess,
            ServiceError::CannotRevokeOwnAdministration,
            ServiceError::ModuleUnknown { key: "x".into() },
            ServiceError::ModuleDisabled { module: "x".into() },
            ServiceError::Module {
                kind: ModuleErrorKind::Conflict,
                code: "conflict.x",
                message: "x".into(),
            },
            ServiceError::TargetNotAccepted {
                module: "x".into(),
                kind: "y".into(),
            },
            ServiceError::TargetRefused {
                code: "tree.not_yet_planted",
                message: "x".into(),
            },
            ServiceError::VehicleCapacityMissing {
                vehicle: uuid::Uuid::nil(),
                kind: "water".into(),
            },
            ServiceError::SettingsEnforcedByAncestor {
                organization_id: uuid::Uuid::nil(),
            },
            ServiceError::OperationPlan(OperationPlanError::InvalidStateTransition {
                from: OperationPlanStatus::Planned,
                to: OperationPlanStatus::Finished,
            }),
            ServiceError::OperationPlan(OperationPlanError::CannotMutateAfterStart),
            ServiceError::OperationPlan(OperationPlanError::CancellationNoteRequired),
            ServiceError::OperationPlan(OperationPlanError::OutcomeMissing(StopRef::new(
                uuid::Uuid::nil(),
            ))),
            ServiceError::OperationPlan(OperationPlanError::OutcomeDuplicate(StopRef::new(
                uuid::Uuid::nil(),
            ))),
            ServiceError::OperationPlan(OperationPlanError::OutcomeUnknownStop(StopRef::new(
                uuid::Uuid::nil(),
            ))),
            ServiceError::OperationPlan(OperationPlanError::TransporterCapacityRequired),
            ServiceError::UnknownStop {
                stop: uuid::Uuid::nil(),
            },
        ]
    }

    #[test]
    fn every_error_carries_a_namespaced_code() {
        for error in every_variant() {
            let code = error.code();
            assert!(
                code.contains('.'),
                "code for {error:?} must be namespaced, got {code:?}"
            );
            assert!(
                code.chars()
                    .all(|c| c.is_ascii_lowercase() || c == '.' || c == '_'),
                "code for {error:?} must be lowercase snake_case, got {code:?}"
            );
        }
    }

    #[test]
    fn distinct_causes_carry_distinct_codes() {
        // The 500 bucket is deliberately shared: the client is not told
        // whether the server tripped over its own data or a driver, and the
        // HTTP layer already collapses both into one generic message.
        let codes: Vec<&'static str> = every_variant()
            .iter()
            .map(ServiceError::code)
            .filter(|c| *c != "internal.error")
            .collect();
        let unique: HashSet<&&str> = codes.iter().collect();
        assert_eq!(
            unique.len(),
            codes.len(),
            "each cause needs its own code, got {codes:?}"
        );
    }

    #[test]
    fn only_the_server_side_failures_share_the_internal_code() {
        let internal: Vec<String> = every_variant()
            .into_iter()
            .filter(|e| e.code() == "internal.error")
            .map(|e| format!("{e:?}"))
            .collect();
        assert_eq!(
            internal.len(),
            2,
            "only DataIntegrity and Internal may collapse, got {internal:?}"
        );
    }
}
