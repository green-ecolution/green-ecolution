//! Region aggregate — a named geographic boundary used to group tree clusters.
//!
//! Follows the standard pattern: [`Region`] aggregate, [`RegionDraft`] for
//! creation, a `pub(crate)` snapshot for DB rehydration, and a
//! [`RegionReader`] / [`RegionWriter`] trait split. No view type is needed
//! because `Region` is small enough to serve as its own read model.

pub mod error;
pub mod repository;
pub mod snapshot;

use crate::{Id, shared::provenance::ProviderId};

pub use error::RegionError;
pub use repository::{RegionReader, RegionWriter};
#[doc(hidden)]
pub use snapshot::RegionSnapshot;

crate::newtype_nonempty! {
    /// Region name, 1–255 characters after trimming.
    RegionName, "region.name", 1, 255
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Region {
    pub id: Id<Region>,
    pub name: RegionName,
}

/// Input for creating a new [`Region`].
#[derive(Debug, Clone)]
pub struct RegionDraft {
    pub name: RegionName,
}

impl Region {
    #[doc(hidden)]
    pub fn reconstitute(snap: RegionSnapshot) -> Self {
        Self {
            id: Id::new(snap.id),
            name: RegionName::reconstitute(snap.name),
        }
    }

    pub fn rename(&mut self, new_name: RegionName) {
        if self.name == new_name {
            return;
        }
        self.name = new_name;
    }
}

#[derive(Debug, Default, Clone)]
pub struct RegionSearchQuery {
    pub provider: Option<ProviderId>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    fn fixed_region() -> Region {
        Region {
            id: Id::new_v7(),
            name: RegionName::new("Flensburg").unwrap(),
        }
    }

    #[test]
    fn region_name_rejects_empty() {
        assert_err!(RegionName::new(""));
    }

    #[test]
    fn region_name_accepts_valid() {
        assert_ok!(RegionName::new("Schleswig"));
    }

    #[test]
    fn rename_to_same_name_is_noop() {
        let mut r = fixed_region();
        r.rename(RegionName::new("Flensburg").unwrap());
        assert_eq!(r.name.as_str(), "Flensburg");
    }

    #[test]
    fn rename_to_new_name_changes_name() {
        let mut r = fixed_region();
        r.rename(RegionName::new("Husum").unwrap());
        assert_eq!(r.name.as_str(), "Husum");
    }
}
