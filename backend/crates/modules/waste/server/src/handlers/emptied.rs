use std::sync::Arc;

use domain::{Id, events::DomainEvent};
use server::service::event_bus::{EventHandler, EventHandlerError};
use waste_domain::waste_bin::{WasteBinReader, WasteBinWriter};

/// Marks the bins a finished collection tour completed as emptied at the tour's finish time.
pub struct BinsEmptied {
    reader: Arc<dyn WasteBinReader>,
    writer: Arc<dyn WasteBinWriter>,
}

impl BinsEmptied {
    pub fn new(reader: Arc<dyn WasteBinReader>, writer: Arc<dyn WasteBinWriter>) -> Self {
        Self { reader, writer }
    }
}

#[async_trait::async_trait]
impl EventHandler for BinsEmptied {
    fn name(&self) -> &str {
        "waste_bins_emptied"
    }

    async fn handle(&self, event: &DomainEvent) -> Result<Vec<DomainEvent>, EventHandlerError> {
        let DomainEvent::OperationPlanFinished {
            module,
            finished_at,
            outcomes,
            ..
        } = event
        else {
            return Ok(vec![]);
        };
        if module.as_str() != crate::KEY {
            return Ok(vec![]);
        }
        for outcome in outcomes.iter().filter(|o| o.completed) {
            let id = Id::new(outcome.stop.value());
            let result = async {
                let mut bin = self.reader.by_id(id).await?;
                bin.mark_emptied(*finished_at);
                self.writer.save_emptied(&bin).await
            }
            .await;
            if let Err(error) = result {
                tracing::warn!(%error, waste_bin.id = %id, "skipping emptied update");
            }
        }
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use chrono::{DateTime, TimeZone, Utc};
    use domain::{
        RepositoryError,
        authorization::Visibility,
        planning::{StopOutcome, StopRef},
        shared::pagination::{Pagination, SearchPage},
    };
    use uuid::Uuid;
    use waste_domain::waste_bin::{
        WasteBin, WasteBinDraft, WasteBinSearchQuery, WasteBinSnapshot, WasteBinView,
    };

    use super::*;

    struct Store(Mutex<Vec<WasteBin>>);

    #[async_trait::async_trait]
    impl WasteBinReader for Store {
        async fn by_id(&self, id: Id<WasteBin>) -> Result<WasteBin, RepositoryError> {
            self.0
                .lock()
                .unwrap()
                .iter()
                .find(|b| b.id == id)
                .cloned()
                .ok_or(RepositoryError::NotFound)
        }
        async fn by_ids(&self, _ids: &[Id<WasteBin>]) -> Result<Vec<WasteBin>, RepositoryError> {
            unimplemented!()
        }
        async fn view_by_id(&self, _id: Id<WasteBin>) -> Result<WasteBinView, RepositoryError> {
            unimplemented!()
        }
        async fn view_by_ids(
            &self,
            _ids: &[Id<WasteBin>],
        ) -> Result<Vec<WasteBinView>, RepositoryError> {
            unimplemented!()
        }
        async fn view_search(
            &self,
            _query: WasteBinSearchQuery,
            _pagination: Pagination,
            _visibility: &Visibility,
        ) -> Result<SearchPage<WasteBinView>, RepositoryError> {
            unimplemented!()
        }
    }

    #[async_trait::async_trait]
    impl WasteBinWriter for Store {
        async fn save_new(&self, _draft: WasteBinDraft) -> Result<WasteBin, RepositoryError> {
            unimplemented!()
        }
        async fn save(&self, _bin: &WasteBin) -> Result<(), RepositoryError> {
            panic!("the handler must not rewrite the whole bin");
        }
        async fn save_fill_level(&self, _bin: &WasteBin) -> Result<(), RepositoryError> {
            unimplemented!()
        }
        async fn save_emptied(&self, bin: &WasteBin) -> Result<(), RepositoryError> {
            let mut bins = self.0.lock().unwrap();
            let slot = bins.iter_mut().find(|b| b.id == bin.id).unwrap();
            *slot = bin.clone();
            Ok(())
        }
        async fn delete(&self, _id: Id<WasteBin>) -> Result<(), RepositoryError> {
            unimplemented!()
        }
    }

    fn bin() -> WasteBin {
        WasteBin::reconstitute(WasteBinSnapshot {
            id: Uuid::now_v7(),
            number: "AB-1".into(),
            latitude: 54.78,
            longitude: 9.43,
            volume_liters: 240.0,
            depth_cm: 100,
            organization_id: Uuid::now_v7(),
            provider: None,
            additional_informations: None,
            fill_fraction: None,
            fill_measured_at: None,
            last_emptied_at: None,
        })
        .unwrap()
    }

    fn finished(module: &str, at: DateTime<Utc>, outcomes: Vec<(&WasteBin, bool)>) -> DomainEvent {
        DomainEvent::OperationPlanFinished {
            plan_id: Id::new_v7(),
            module: domain::module::ModuleKey::new(module).unwrap(),
            stops: vec![],
            finished_at: at,
            outcomes: outcomes
                .into_iter()
                .map(|(b, completed)| StopOutcome {
                    stop: StopRef::new(b.id.value()),
                    completed,
                    amount: None,
                })
                .collect(),
        }
    }

    fn handler(bins: Vec<WasteBin>) -> (BinsEmptied, Arc<Store>) {
        let store = Arc::new(Store(Mutex::new(bins)));
        (BinsEmptied::new(store.clone(), store.clone()), store)
    }

    fn at() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 2, 12, 0, 0).unwrap()
    }

    #[tokio::test]
    async fn only_completed_stops_are_emptied_at_the_finish_time() {
        let (a, b) = (bin(), bin());
        let (handler, store) = handler(vec![a.clone(), b.clone()]);
        handler
            .handle(&finished("waste", at(), vec![(&a, true), (&b, false)]))
            .await
            .unwrap();
        let bins = store.0.lock().unwrap();
        assert_eq!(bins[0].last_emptied_at(), Some(at()));
        assert_eq!(bins[1].last_emptied_at(), None);
    }

    #[tokio::test]
    async fn a_plan_of_another_module_is_ignored() {
        let a = bin();
        let (handler, store) = handler(vec![a.clone()]);
        handler
            .handle(&finished("irrigation", at(), vec![(&a, true)]))
            .await
            .unwrap();
        assert_eq!(store.0.lock().unwrap()[0].last_emptied_at(), None);
    }

    #[tokio::test]
    async fn a_missing_bin_does_not_stop_the_others() {
        let (ghost, b) = (bin(), bin());
        let (handler, store) = handler(vec![b.clone()]);
        handler
            .handle(&finished("waste", at(), vec![(&ghost, true), (&b, true)]))
            .await
            .unwrap();
        assert_eq!(store.0.lock().unwrap()[0].last_emptied_at(), Some(at()));
    }
}
