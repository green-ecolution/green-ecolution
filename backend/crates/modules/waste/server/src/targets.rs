//! Waste bins as targets the core can point at.

use domain::{Id, target::TargetRef};
use waste_domain::waste_bin::WasteBin;

use super::KEY;

pub const BIN_KIND: &str = "bin";

pub fn bin_target(id: Id<WasteBin>) -> TargetRef {
    TargetRef::new(KEY, BIN_KIND, id.value()).expect("static keys are valid")
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn bin_target_uses_the_waste_keys() {
        let id = Uuid::now_v7();
        let target = bin_target(Id::new(id));
        assert_eq!(target.module.as_str(), "waste");
        assert_eq!(target.kind.as_str(), "bin");
        assert_eq!(target.id, id);
    }
}
