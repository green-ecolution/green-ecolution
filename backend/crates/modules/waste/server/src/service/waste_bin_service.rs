use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Duration, Utc};

use domain::{
    Id, RepositoryError,
    authorization::Visibility,
    organization::Organization,
    planning::StopRef,
    shared::pagination::{Page, Pagination, SearchPage},
};
use server::service::{
    comment_service::CommentService, operation_plan_service::OperationPlanService,
    sensor_service::SensorService,
};
use waste_domain::{
    fill::FillState,
    settings::FullThreshold,
    waste_bin::{
        FillFilter, WasteBin, WasteBinDraft, WasteBinReader, WasteBinSearchQuery, WasteBinUpdate,
        WasteBinView, WasteBinWriter,
    },
};

use super::fill_rules::{FillRules, FillRulesResolver};
use crate::{error::WasteError, targets::bin_target};

/// A bin together with the fill state its organization's rules derive.
#[derive(Debug, Clone)]
pub struct WasteBinState {
    pub view: WasteBinView,
    pub fill_state: FillState,
}

/// Narrows the list to one fill state, judged by the rules of `reference_org`.
#[derive(Debug, Clone, Copy)]
pub struct FillStateFilter {
    pub state: FillState,
    pub reference_org: Id<Organization>,
}

#[derive(Debug, Clone, Default)]
pub struct WasteBinListQuery {
    pub number: Option<String>,
    pub organization_id: Option<Id<Organization>>,
    pub fill_state: Option<FillStateFilter>,
}

pub fn filter_for(
    state: FillState,
    threshold: FullThreshold,
    now: DateTime<Utc>,
    stale_after: Duration,
) -> FillFilter {
    let fresh = Some(now - stale_after);
    match state {
        FillState::Full => FillFilter {
            min_fraction: Some(threshold.fraction()),
            measured_after: fresh,
            require_measurement: true,
            not_emptied_since_measurement: true,
            ..Default::default()
        },
        FillState::Normal => FillFilter {
            max_fraction: Some(threshold.fraction()),
            measured_after: fresh,
            require_measurement: true,
            not_emptied_since_measurement: true,
            ..Default::default()
        },
        FillState::Empty => FillFilter {
            measured_after: fresh,
            require_measurement: true,
            emptied_after_measurement: true,
            ..Default::default()
        },
        FillState::Unknown => FillFilter {
            stale_before: fresh,
            ..Default::default()
        },
    }
}

pub struct WasteBinService {
    reader: Arc<dyn WasteBinReader>,
    writer: Arc<dyn WasteBinWriter>,
    sensors: Arc<SensorService>,
    plans: Arc<OperationPlanService>,
    comments: Arc<CommentService>,
    fill_rules: Arc<FillRulesResolver>,
}

impl WasteBinService {
    pub fn new(
        reader: Arc<dyn WasteBinReader>,
        writer: Arc<dyn WasteBinWriter>,
        sensors: Arc<SensorService>,
        plans: Arc<OperationPlanService>,
        comments: Arc<CommentService>,
        fill_rules: Arc<FillRulesResolver>,
    ) -> Self {
        Self {
            reader,
            writer,
            sensors,
            plans,
            comments,
            fill_rules,
        }
    }

    async fn rules_for(&self, org: Id<Organization>) -> Result<FillRules, WasteError> {
        Ok(self.fill_rules.rules_for(org).await?)
    }

    /// Resolves the rules once per organization, however many bins it owns.
    async fn with_states(
        &self,
        views: Vec<WasteBinView>,
    ) -> Result<Vec<WasteBinState>, WasteError> {
        let now = Utc::now();
        let mut rules: HashMap<Id<Organization>, FillRules> = HashMap::new();
        let mut out = Vec::with_capacity(views.len());
        for view in views {
            let org_rules = match rules.get(&view.organization_id) {
                Some(r) => *r,
                None => {
                    let r = self.rules_for(view.organization_id).await?;
                    rules.insert(view.organization_id, r);
                    r
                }
            };
            let fill_state = waste_domain::fill::derive_fill_state(
                view.fill,
                view.last_emptied_at,
                org_rules.threshold,
                now,
                org_rules.stale_after,
            );
            out.push(WasteBinState { view, fill_state });
        }
        Ok(out)
    }

    async fn state_of(&self, id: Id<WasteBin>) -> Result<WasteBinState, WasteError> {
        let view = self.reader.view_by_id(id).await?;
        let mut states = self.with_states(vec![view]).await?;
        Ok(states.remove(0))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn list(
        &self,
        query: WasteBinListQuery,
        pagination: Pagination,
        visibility: &Visibility,
    ) -> Result<SearchPage<WasteBinState>, WasteError> {
        let fill = match query.fill_state {
            Some(filter) => {
                let rules = self.rules_for(filter.reference_org).await?;
                Some(filter_for(
                    filter.state,
                    rules.threshold,
                    Utc::now(),
                    rules.stale_after,
                ))
            }
            None => None,
        };
        let search = WasteBinSearchQuery {
            number: query.number,
            organization_id: query.organization_id,
            fill,
        };
        let result = self
            .reader
            .view_search(search, pagination, visibility)
            .await?;
        Ok(SearchPage {
            page: Page {
                items: self.with_states(result.page.items).await?,
                total: result.page.total,
            },
            total_unfiltered: result.total_unfiltered,
        })
    }

    #[tracing::instrument(level = "debug", skip_all, fields(waste_bin.id = %id))]
    pub async fn get(&self, id: Id<WasteBin>) -> Result<WasteBinState, WasteError> {
        self.state_of(id).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn create(&self, draft: WasteBinDraft) -> Result<WasteBinState, WasteError> {
        let bin = self.writer.save_new(draft).await.map_err(number_taken)?;
        self.state_of(bin.id).await
    }

    #[tracing::instrument(level = "debug", skip_all, fields(waste_bin.id = %id))]
    pub async fn update(
        &self,
        id: Id<WasteBin>,
        update: WasteBinUpdate,
    ) -> Result<WasteBinState, WasteError> {
        let mut bin = self.reader.by_id(id).await?;
        bin.replace_details(update);
        self.writer.save(&bin).await.map_err(number_taken)?;
        self.state_of(id).await
    }

    #[tracing::instrument(level = "debug", skip_all, fields(waste_bin.id = %id))]
    pub async fn delete(&self, id: Id<WasteBin>) -> Result<(), WasteError> {
        self.reader.by_id(id).await?;
        let target = bin_target(id);
        self.sensors.release_target(&target).await?;
        self.plans
            .remove_stop(&crate::key(), StopRef::new(id.value()))
            .await?;
        self.writer.delete(id).await?;
        // Comments have no cascading FK: the subject is polymorphic.
        let deleted_comments = self.comments.delete_for_target(&target).await?;
        tracing::debug!(deleted_comments, "removed comments for deleted waste bin");
        Ok(())
    }

    /// An attached sensor moves along, since a sensor and its target must
    /// share an organization.
    #[tracing::instrument(level = "debug", skip_all, fields(waste_bin.id = %id))]
    pub async fn transfer(
        &self,
        id: Id<WasteBin>,
        target: Id<Organization>,
    ) -> Result<(), WasteError> {
        let mut bin = self.reader.by_id(id).await?;
        bin.transfer_to(target);
        self.writer.save(&bin).await.map_err(number_taken)?;
        if let Some(sensor_id) = self.reader.view_by_id(id).await?.sensor_id {
            self.sensors
                .transfer_with_target(&sensor_id, target)
                .await?;
        }
        Ok(())
    }
}

fn number_taken(e: RepositoryError) -> WasteError {
    match e {
        RepositoryError::AlreadyExists(_) => WasteError::NumberTaken,
        e => e.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 2, 8, 0, 0).unwrap()
    }

    fn threshold() -> FullThreshold {
        FullThreshold::new(80).unwrap()
    }

    fn filter(state: FillState) -> FillFilter {
        filter_for(state, threshold(), now(), Duration::hours(6))
    }

    #[test]
    fn full_needs_a_fresh_reading_at_or_above_the_threshold_without_emptying() {
        assert_eq!(
            filter(FillState::Full),
            FillFilter {
                min_fraction: Some(0.8),
                max_fraction: None,
                measured_after: Some(now() - Duration::hours(6)),
                stale_before: None,
                require_measurement: true,
                not_emptied_since_measurement: true,
                emptied_after_measurement: false,
            }
        );
    }

    #[test]
    fn normal_needs_a_fresh_reading_below_the_threshold_without_emptying() {
        assert_eq!(
            filter(FillState::Normal),
            FillFilter {
                min_fraction: None,
                max_fraction: Some(0.8),
                measured_after: Some(now() - Duration::hours(6)),
                stale_before: None,
                require_measurement: true,
                not_emptied_since_measurement: true,
                emptied_after_measurement: false,
            }
        );
    }

    #[test]
    fn empty_needs_a_fresh_reading_followed_by_an_emptying() {
        assert_eq!(
            filter(FillState::Empty),
            FillFilter {
                measured_after: Some(now() - Duration::hours(6)),
                require_measurement: true,
                emptied_after_measurement: true,
                ..Default::default()
            }
        );
    }

    #[test]
    fn unknown_matches_missing_or_stale_readings() {
        assert_eq!(
            filter(FillState::Unknown),
            FillFilter {
                stale_before: Some(now() - Duration::hours(6)),
                ..Default::default()
            }
        );
    }

    #[test]
    fn a_unique_violation_is_a_taken_number() {
        assert!(matches!(
            number_taken(RepositoryError::AlreadyExists("x".into())),
            WasteError::NumberTaken
        ));
        assert!(matches!(
            number_taken(RepositoryError::NotFound),
            WasteError::Core(_)
        ));
    }
}
