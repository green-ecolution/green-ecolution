use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
};

use chrono::{DateTime, Utc};

use domain::{
    Id, RepositoryError,
    authorization::Visibility,
    organization::{Organization, OrganizationReader},
    planning::StopRef,
    shared::pagination::{Page, Pagination, SearchPage},
};
use server::service::{
    comment_service::CommentService, operation_plan_service::OperationPlanService,
    sensor_service::SensorService,
};
use waste_domain::{
    fill::FillState,
    waste_bin::{
        FillFilter, OrganizationFillBounds, WasteBin, WasteBinDraft, WasteBinReader,
        WasteBinSearchQuery, WasteBinUpdate, WasteBinView, WasteBinWriter,
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

#[derive(Debug, Clone, Default)]
pub struct WasteBinListQuery {
    pub number: Option<String>,
    pub organization_id: Option<Id<Organization>>,
    /// Judged per bin by the rules of the bin's own organization, so the
    /// filter and the `fill_state` label of a listed bin always agree.
    pub fill_state: Option<FillState>,
}

pub fn filter_for(
    state: FillState,
    now: DateTime<Utc>,
    rules: &[(Id<Organization>, FillRules)],
) -> FillFilter {
    let bounds = rules
        .iter()
        .map(|(org, rules)| {
            let fresh = Some(now - rules.stale_after);
            let threshold = Some(rules.threshold.fraction());
            let (min_fraction, max_fraction, measured_after, stale_before) = match state {
                FillState::Full => (threshold, None, fresh, None),
                FillState::Normal => (None, threshold, fresh, None),
                FillState::Empty => (None, None, fresh, None),
                FillState::Unknown => (None, None, None, fresh),
            };
            OrganizationFillBounds {
                organization_id: *org,
                min_fraction,
                max_fraction,
                measured_after,
                stale_before,
            }
        })
        .collect();
    let (require_measurement, not_emptied_since_measurement, emptied_after_measurement) =
        match state {
            FillState::Full | FillState::Normal => (true, true, false),
            FillState::Empty => (true, false, true),
            FillState::Unknown => (false, false, false),
        };
    FillFilter {
        bounds,
        require_measurement,
        not_emptied_since_measurement,
        emptied_after_measurement,
    }
}

pub struct WasteBinService {
    reader: Arc<dyn WasteBinReader>,
    writer: Arc<dyn WasteBinWriter>,
    sensors: Arc<SensorService>,
    plans: Arc<OperationPlanService>,
    comments: Arc<CommentService>,
    fill_rules: Arc<FillRulesResolver>,
    organizations: Arc<dyn OrganizationReader>,
}

impl WasteBinService {
    pub fn new(
        reader: Arc<dyn WasteBinReader>,
        writer: Arc<dyn WasteBinWriter>,
        sensors: Arc<SensorService>,
        plans: Arc<OperationPlanService>,
        comments: Arc<CommentService>,
        fill_rules: Arc<FillRulesResolver>,
        organizations: Arc<dyn OrganizationReader>,
    ) -> Self {
        Self {
            reader,
            writer,
            sensors,
            plans,
            comments,
            fill_rules,
            organizations,
        }
    }

    /// Resolves the rules once per organization, however many bins it owns.
    /// Takes the ids by value: a borrowed iterator held across the awaits
    /// makes the handler future non-`Send` for axum.
    async fn rules_by_org(
        &self,
        orgs: Vec<Id<Organization>>,
    ) -> Result<HashMap<Id<Organization>, FillRules>, WasteError> {
        let mut rules = HashMap::new();
        for org in orgs {
            if let Entry::Vacant(slot) = rules.entry(org) {
                slot.insert(self.fill_rules.rules_for(org).await?);
            }
        }
        Ok(rules)
    }

    async fn with_states(
        &self,
        views: Vec<WasteBinView>,
    ) -> Result<Vec<WasteBinState>, WasteError> {
        let now = Utc::now();
        let rules = self
            .rules_by_org(views.iter().map(|v| v.organization_id).collect())
            .await?;
        Ok(views
            .into_iter()
            .map(|view| {
                let org_rules = rules[&view.organization_id];
                let fill_state = waste_domain::fill::derive_fill_state(
                    view.fill,
                    view.last_emptied_at,
                    org_rules.threshold,
                    now,
                    org_rules.stale_after,
                );
                WasteBinState { view, fill_state }
            })
            .collect())
    }

    /// The organizations a list request can return bins from.
    async fn candidate_orgs(
        &self,
        visibility: &Visibility,
        requested: Option<Id<Organization>>,
    ) -> Result<Vec<Id<Organization>>, WasteError> {
        let visible: Vec<Id<Organization>> = match visibility {
            Visibility::Only(orgs) => orgs.iter().copied().collect(),
            Visibility::Unrestricted => self
                .organizations
                .all()
                .await?
                .into_iter()
                .map(|o| o.id)
                .collect(),
        };
        Ok(visible
            .into_iter()
            .filter(|org| requested.is_none_or(|r| r == *org))
            .collect())
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
            Some(state) => {
                let orgs = self
                    .candidate_orgs(visibility, query.organization_id)
                    .await?;
                let rules = self.rules_by_org(orgs.clone()).await?;
                let rules: Vec<(Id<Organization>, FillRules)> =
                    orgs.into_iter().map(|org| (org, rules[&org])).collect();
                Some(filter_for(state, Utc::now(), &rules))
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
    use chrono::{Duration, TimeZone};
    use waste_domain::settings::FullThreshold;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 2, 8, 0, 0).unwrap()
    }

    fn org() -> Id<Organization> {
        Id::new(uuid::Uuid::from_u128(1))
    }

    fn rules(percent: i64) -> FillRules {
        FillRules {
            threshold: FullThreshold::new(percent).unwrap(),
            stale_after: Duration::hours(6),
        }
    }

    fn filter(state: FillState) -> FillFilter {
        filter_for(state, now(), &[(org(), rules(80))])
    }

    fn bounds(
        min_fraction: Option<f64>,
        max_fraction: Option<f64>,
        measured_after: Option<DateTime<Utc>>,
        stale_before: Option<DateTime<Utc>>,
    ) -> Vec<OrganizationFillBounds> {
        vec![OrganizationFillBounds {
            organization_id: org(),
            min_fraction,
            max_fraction,
            measured_after,
            stale_before,
        }]
    }

    #[test]
    fn full_needs_a_fresh_reading_at_or_above_the_threshold_without_emptying() {
        assert_eq!(
            filter(FillState::Full),
            FillFilter {
                bounds: bounds(Some(0.8), None, Some(now() - Duration::hours(6)), None),
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
                bounds: bounds(None, Some(0.8), Some(now() - Duration::hours(6)), None),
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
                bounds: bounds(None, None, Some(now() - Duration::hours(6)), None),
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
                bounds: bounds(None, None, None, Some(now() - Duration::hours(6))),
                ..Default::default()
            }
        );
    }

    #[test]
    fn every_organization_gets_its_own_threshold() {
        let other = Id::new(uuid::Uuid::from_u128(2));
        let filter = filter_for(
            FillState::Full,
            now(),
            &[(org(), rules(80)), (other, rules(90))],
        );
        let thresholds: Vec<(Id<Organization>, Option<f64>)> = filter
            .bounds
            .iter()
            .map(|b| (b.organization_id, b.min_fraction))
            .collect();
        assert_eq!(thresholds, [(org(), Some(0.8)), (other, Some(0.9))]);
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
