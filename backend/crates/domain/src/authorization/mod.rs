//! Authorization value objects: the compile-time permission catalog and the
//! request-scoped evaluation types (`EffectivePermissions`, `AccessContext`).

pub mod effective;
pub mod permission;
pub mod resource;

pub use effective::{AccessContext, Caller, EffectivePermissions, OrgHierarchy, Visibility};
pub use permission::{Action, Permission};
pub use resource::{PermissionParseError, ResourceCatalog, ResourceKey};
