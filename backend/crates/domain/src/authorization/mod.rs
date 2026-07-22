//! Authorization value objects: the compile-time permission catalog and the
//! request-scoped evaluation types (`EffectivePermissions`, `AccessContext`).

pub mod permission;

pub use permission::{Action, Permission, Resource};
