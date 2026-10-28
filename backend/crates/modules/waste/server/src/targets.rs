//! Waste bins as targets the core can point at.

use std::{collections::HashMap, sync::Arc};

use domain::{
    Id, RepositoryError,
    organization::Organization,
    shared::coordinates::Coordinate,
    target::{TargetCatalog, TargetError, TargetKindSpec, TargetPurpose, TargetRef},
};
use waste_domain::waste_bin::{WasteBin, WasteBinReader};

use super::{KEY, resources};

pub const BIN_KIND: &str = "bin";

const KINDS: [TargetKindSpec; 1] = [TargetKindSpec {
    kind: BIN_KIND,
    accepts_sensors: true,
    commentable: true,
    resource: resources::WASTE_BIN,
}];

pub fn bin_target(id: Id<WasteBin>) -> TargetRef {
    TargetRef::new(KEY, BIN_KIND, id.value()).expect("static keys are valid")
}

pub fn is_bin(target: &TargetRef) -> bool {
    target.module.as_str() == KEY && target.kind.as_str() == BIN_KIND
}

pub struct WasteTargets {
    bins: Arc<dyn WasteBinReader>,
}

impl WasteTargets {
    pub fn new(bins: Arc<dyn WasteBinReader>) -> Self {
        Self { bins }
    }

    async fn bins_of(&self, targets: &[TargetRef]) -> Result<Vec<WasteBin>, RepositoryError> {
        let ids: Vec<Id<WasteBin>> = targets
            .iter()
            .filter(|t| is_bin(t))
            .map(|t| Id::new(t.id))
            .collect();
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        self.bins.by_ids(&ids).await
    }
}

#[async_trait::async_trait]
impl TargetCatalog for WasteTargets {
    fn kinds(&self) -> &'static [TargetKindSpec] {
        &KINDS
    }

    async fn admit(
        &self,
        target: &TargetRef,
        _purpose: TargetPurpose,
    ) -> Result<Id<Organization>, TargetError> {
        if !is_bin(target) {
            return Err(TargetError::NotFound);
        }
        match self.bins.by_id(Id::new(target.id)).await {
            Ok(bin) => Ok(bin.organization_id()),
            Err(RepositoryError::NotFound) => Err(TargetError::NotFound),
            Err(e) => Err(TargetError::Repository(e)),
        }
    }

    async fn locate(
        &self,
        targets: &[TargetRef],
    ) -> Result<HashMap<TargetRef, Coordinate>, RepositoryError> {
        Ok(self
            .bins_of(targets)
            .await?
            .into_iter()
            .map(|bin| (bin_target(bin.id), bin.coordinate))
            .collect())
    }

    async fn label(
        &self,
        targets: &[TargetRef],
    ) -> Result<HashMap<TargetRef, String>, RepositoryError> {
        Ok(self
            .bins_of(targets)
            .await?
            .into_iter()
            .map(|bin| (bin_target(bin.id), bin.number.as_str().to_owned()))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use domain::{
        authorization::Visibility,
        shared::pagination::{Pagination, SearchPage},
    };
    use uuid::Uuid;
    use waste_domain::waste_bin::{WasteBinSearchQuery, WasteBinSnapshot, WasteBinView};

    fn bin(number: &str) -> WasteBin {
        WasteBin::reconstitute(WasteBinSnapshot {
            id: Uuid::now_v7(),
            number: number.into(),
            latitude: 54.79,
            longitude: 9.44,
            volume_liters: 1100.0,
            depth_cm: 120,
            organization_id: Uuid::now_v7(),
            provider: None,
            additional_informations: None,
            fill_fraction: None,
            fill_measured_at: None,
            last_emptied_at: None,
        })
        .unwrap()
    }

    struct Bins(Vec<WasteBin>);

    #[async_trait]
    impl WasteBinReader for Bins {
        async fn by_id(&self, id: Id<WasteBin>) -> Result<WasteBin, RepositoryError> {
            self.0
                .iter()
                .find(|b| b.id == id)
                .cloned()
                .ok_or(RepositoryError::NotFound)
        }
        async fn by_ids(&self, ids: &[Id<WasteBin>]) -> Result<Vec<WasteBin>, RepositoryError> {
            Ok(self
                .0
                .iter()
                .filter(|b| ids.contains(&b.id))
                .cloned()
                .collect())
        }
        async fn view_by_id(&self, _: Id<WasteBin>) -> Result<WasteBinView, RepositoryError> {
            unimplemented!()
        }
        async fn view_by_ids(
            &self,
            _: &[Id<WasteBin>],
        ) -> Result<Vec<WasteBinView>, RepositoryError> {
            unimplemented!()
        }
        async fn view_search(
            &self,
            _: WasteBinSearchQuery,
            _: Pagination,
            _: &Visibility,
        ) -> Result<SearchPage<WasteBinView>, RepositoryError> {
            unimplemented!()
        }
    }

    fn catalog(bins: Vec<WasteBin>) -> WasteTargets {
        WasteTargets::new(Arc::new(Bins(bins)))
    }

    #[tokio::test]
    async fn admit_returns_the_organization_of_the_bin() {
        let b = bin("AB-1");
        let org = b.organization_id();
        let id = b.id;
        let got = catalog(vec![b])
            .admit(&bin_target(id), TargetPurpose::Sensor)
            .await
            .unwrap();
        assert_eq!(got, org);
    }

    #[tokio::test]
    async fn admit_rejects_an_unknown_bin() {
        let err = catalog(vec![])
            .admit(&bin_target(Id::new_v7()), TargetPurpose::Sensor)
            .await
            .unwrap_err();
        assert!(matches!(err, TargetError::NotFound));
    }

    #[tokio::test]
    async fn label_is_the_bin_number_and_locate_its_coordinate() {
        let b = bin("AB-1");
        let target = bin_target(b.id);
        let c = catalog(vec![b]);
        let labels = c.label(std::slice::from_ref(&target)).await.unwrap();
        assert_eq!(labels[&target], "AB-1");
        assert_eq!(
            c.locate(std::slice::from_ref(&target)).await.unwrap().len(),
            1
        );
    }

    #[test]
    fn bins_take_sensors_and_comments() {
        let spec = KINDS.iter().find(|k| k.kind == BIN_KIND).unwrap();
        assert!(spec.allows(TargetPurpose::Sensor) && spec.allows(TargetPurpose::Comment));
    }

    #[test]
    fn bin_target_uses_the_waste_keys() {
        let id = Uuid::now_v7();
        let target = bin_target(Id::new(id));
        assert_eq!(target.module.as_str(), "waste");
        assert_eq!(target.kind.as_str(), "bin");
        assert_eq!(target.id, id);
    }
}
