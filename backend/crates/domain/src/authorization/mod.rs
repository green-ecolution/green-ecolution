//! Authorization value objects: the compile-time permission catalog and the
//! request-scoped evaluation types (`EffectivePermissions`, `AccessContext`).

pub mod effective;
pub mod permission;

pub use effective::{AccessContext, EffectivePermissions, OrgHierarchy};
pub use permission::{Action, Permission, Resource};
