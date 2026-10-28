use std::sync::Arc;

use domain::{
    Id,
    authorization::{OrgHierarchy, Visibility},
    capacity::{CapacityKind, Liters, route_capacity},
    comment::{CommentSubject, CommentWriter},
    events::DomainEvent,
    module::{ModuleActivationReader, ModuleKey},
    operation_plan::{
        OperationPlan, OperationPlanDraft, OperationPlanError, OperationPlanReader,
        OperationPlanSearchQuery, OperationPlanUpdate, OperationPlanView, OperationPlanWriter,
        RouteMetrics, StationVisit, ensure_capacity_available,
    },
    organization::{Organization, OrganizationReader},
    planning::{PlanningError, PlanningProvider, ResolvedStop, StopOutcome, StopRef},
    routing::{OptimizedRoute, RouteOptimizer, RouteRequest, RouteStop, RoutingError},
    shared::{
        coordinates::Coordinate,
        pagination::{Page, Pagination},
    },
    start_point::{StartPoint, StartPointReader},
    vehicle::{Vehicle, VehicleReader},
};

use super::{
    Feature, OrganizationMismatch, ServiceError, event_bus::EventBus, planning::PlanningRegistry,
    targets::ensure_module_active,
};

pub struct OperationPlanService {
    reader: Arc<dyn OperationPlanReader>,
    writer: Arc<dyn OperationPlanWriter>,
    vehicle_reader: Arc<dyn VehicleReader>,
    event_bus: Arc<dyn EventBus>,
    route_optimizer: Option<Arc<dyn RouteOptimizer>>,
    start_point_reader: Arc<dyn StartPointReader>,
    comment_writer: Arc<dyn CommentWriter>,
    planning: Arc<PlanningRegistry>,
    activation: Arc<dyn ModuleActivationReader>,
    organizations: Arc<dyn OrganizationReader>,
}

/// Input for a route computation that is not (yet) a stored plan.
#[derive(Debug, Clone)]
pub struct PreviewRoute {
    pub module: ModuleKey,
    pub stops: Vec<StopRef>,
    pub transporter_id: Id<Vehicle>,
    pub trailer_id: Option<Id<Vehicle>>,
    pub transporter_capacity_used: bool,
    pub start_point_name: Option<String>,
    pub organization_id: Id<Organization>,
}

/// Result of a route computation: the optimized route plus the summed stop
/// demand (which the solver does not report back).
pub struct ComputedRoute {
    pub route: OptimizedRoute,
    pub total_demand: f64,
    pub station_visits: Vec<StationVisit>,
}

impl OperationPlanService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        reader: Arc<dyn OperationPlanReader>,
        writer: Arc<dyn OperationPlanWriter>,
        vehicle_reader: Arc<dyn VehicleReader>,
        event_bus: Arc<dyn EventBus>,
        route_optimizer: Option<Arc<dyn RouteOptimizer>>,
        start_point_reader: Arc<dyn StartPointReader>,
        comment_writer: Arc<dyn CommentWriter>,
        planning: Arc<PlanningRegistry>,
        activation: Arc<dyn ModuleActivationReader>,
        organizations: Arc<dyn OrganizationReader>,
    ) -> Self {
        Self {
            reader,
            writer,
            vehicle_reader,
            event_bus,
            route_optimizer,
            start_point_reader,
            comment_writer,
            planning,
            activation,
            organizations,
        }
    }

    async fn resolve(
        &self,
        provider: &dyn PlanningProvider,
        stops: &[StopRef],
        org: Id<Organization>,
    ) -> Result<Vec<ResolvedStop>, ServiceError> {
        provider
            .resolve_stops(stops, org)
            .await
            .map_err(|e| match e {
                PlanningError::OrganizationMismatch => OrganizationMismatch::StopsVsPlan.into(),
                PlanningError::UnknownStop(stop) => {
                    ServiceError::UnknownStop { stop: stop.value() }
                }
                PlanningError::Repository(e) => e.into(),
            })
    }

    /// Refuses a plan whose vehicles lack the capacity the plan relies on,
    /// whether or not routing is enabled.
    async fn check_vehicles(
        &self,
        kind: CapacityKind,
        transporter_id: Option<Id<Vehicle>>,
        trailer_id: Option<Id<Vehicle>>,
        capacity_used: bool,
    ) -> Result<(), ServiceError> {
        let Some(transporter_id) = transporter_id else {
            return Ok(());
        };
        let transporter = self.vehicle_reader.by_id(transporter_id).await?;
        let trailer = match trailer_id {
            Some(id) => Some(self.vehicle_reader.by_id(id).await?),
            None => None,
        };
        capacity(kind, &transporter, trailer.as_ref(), capacity_used)?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn search_view(
        &self,
        query: OperationPlanSearchQuery,
        pagination: Pagination,
    ) -> Result<Page<OperationPlanView>, ServiceError> {
        Ok(self.reader.view_search(query, pagination).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plan.id = %id))]
    pub async fn by_id(&self, id: Id<OperationPlan>) -> Result<OperationPlan, ServiceError> {
        Ok(self.reader.by_id(id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plan.id = %id))]
    pub async fn view_by_id(
        &self,
        id: Id<OperationPlan>,
    ) -> Result<OperationPlanView, ServiceError> {
        Ok(self.reader.view_by_id(id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plan.id = %id))]
    pub async fn outcomes(&self, id: Id<OperationPlan>) -> Result<Vec<StopOutcome>, ServiceError> {
        Ok(self.reader.outcomes(id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(module = %draft.module))]
    pub async fn create(&self, draft: OperationPlanDraft) -> Result<OperationPlan, ServiceError> {
        let provider = self.planning.provider(&draft.module)?;
        ensure_module_active(
            self.activation.as_ref(),
            &draft.module,
            draft.organization_id,
        )
        .await?;
        ensure_capacity_available(draft.trailer_id, draft.transporter_capacity_used)?;
        self.resolve(provider.as_ref(), &draft.stops, draft.organization_id)
            .await?;
        self.check_vehicles(
            provider.capacity_kind(),
            draft.transporter_id,
            draft.trailer_id,
            draft.transporter_capacity_used,
        )
        .await?;
        let mut plan = self.writer.save_new(draft).await?;
        if self.route_optimizer.is_some() {
            self.apply_route(&mut plan).await;
        }
        Ok(plan)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plan.id = %id))]
    pub async fn replace_details(
        &self,
        id: Id<OperationPlan>,
        update: OperationPlanUpdate,
    ) -> Result<OperationPlan, ServiceError> {
        let mut plan = self.reader.by_id(id).await?;
        let provider = self.planning.provider(plan.module())?;
        ensure_module_active(
            self.activation.as_ref(),
            plan.module(),
            plan.organization_id(),
        )
        .await?;
        self.resolve(provider.as_ref(), &update.stops, plan.organization_id())
            .await?;
        plan.replace_details(update)?;
        self.check_vehicles(
            provider.capacity_kind(),
            plan.transporter_id(),
            plan.trailer_id(),
            plan.transporter_capacity_used(),
        )
        .await?;
        if self.route_optimizer.is_some() {
            // Edited stop/vehicle set invalidates the old route; a failed
            // recompute must leave the "no route" state, not a stale track.
            plan.set_metrics(RouteMetrics::cleared());
        }
        self.writer.save(&plan).await?;
        if self.route_optimizer.is_some() {
            self.apply_route(&mut plan).await;
        }
        Ok(plan)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plan.id = %id))]
    pub async fn start(&self, id: Id<OperationPlan>) -> Result<OperationPlan, ServiceError> {
        self.transition(id, |plan| plan.start()).await
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plan.id = %id))]
    pub async fn revert_start(&self, id: Id<OperationPlan>) -> Result<OperationPlan, ServiceError> {
        self.transition(id, |plan| plan.revert_start()).await
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plan.id = %id))]
    pub async fn cancel(
        &self,
        id: Id<OperationPlan>,
        note: String,
    ) -> Result<OperationPlan, ServiceError> {
        self.transition(id, move |plan| plan.cancel(note)).await
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plan.id = %id))]
    pub async fn fail(
        &self,
        id: Id<OperationPlan>,
        note: String,
    ) -> Result<OperationPlan, ServiceError> {
        self.transition(id, move |plan| plan.fail(note)).await
    }

    #[tracing::instrument(level = "debug", skip_all, fields(module = %req.module))]
    pub async fn preview_route(&self, req: PreviewRoute) -> Result<ComputedRoute, ServiceError> {
        ensure_module_active(self.activation.as_ref(), &req.module, req.organization_id).await?;
        ensure_capacity_available(req.trailer_id, req.transporter_capacity_used)?;
        self.compute_route(&req).await
    }

    /// Route failures must never block plan persistence: the plan is already
    /// saved and the frontend surfaces a missing route via `distance == 0`.
    async fn apply_route(&self, plan: &mut OperationPlan) {
        let Some(transporter_id) = plan.transporter_id() else {
            return;
        };
        let req = PreviewRoute {
            module: plan.module().clone(),
            stops: plan.stops().to_vec(),
            transporter_id,
            trailer_id: plan.trailer_id(),
            transporter_capacity_used: plan.transporter_capacity_used(),
            start_point_name: plan.start_point_name.clone(),
            organization_id: plan.organization_id(),
        };
        match self.compute_route(&req).await {
            Ok(computed) => {
                plan.set_metrics(RouteMetrics {
                    distance: Some(computed.route.distance),
                    total_demand: Some(computed.total_demand),
                    refill_count: computed.route.refill_count,
                    duration: computed.route.duration,
                    route_geometry: Some(computed.route.geometry),
                    station_visits: computed.station_visits,
                });
                if let Err(e) = self.writer.save(plan).await {
                    tracing::warn!(error = %e, "failed to persist route metrics");
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "route optimization failed; plan saved without route");
            }
        }
    }

    async fn compute_route(&self, req: &PreviewRoute) -> Result<ComputedRoute, ServiceError> {
        let optimizer = self
            .route_optimizer
            .as_ref()
            .ok_or(ServiceError::FeatureDisabled {
                feature: Feature::Routing,
            })?;
        let provider = self.planning.provider(&req.module)?;
        let kind = provider.capacity_kind();
        let resolved = self
            .resolve(provider.as_ref(), &req.stops, req.organization_id)
            .await?;
        let stops: Vec<RouteStop> = resolved
            .iter()
            .filter_map(|r| {
                r.location.map(|location| RouteStop {
                    stop: r.stop,
                    location,
                    demand: r.demand,
                })
            })
            .collect();
        if stops.is_empty() {
            return Err(ServiceError::InvalidInput(
                "no stop with a position to route".into(),
            ));
        }
        let transporter = self.vehicle_reader.by_id(req.transporter_id).await?;
        let trailer = match req.trailer_id {
            Some(id) => Some(self.vehicle_reader.by_id(id).await?),
            None => None,
        };
        // Internal lookup for the plan's own depot selection, not a user-facing
        // listing — access to the plan itself is already gated by the caller.
        let hierarchy = self.organizations.hierarchy().await?;
        let ranked_orgs = related_orgs_nearest_first(&hierarchy, req.organization_id);
        let start_points = self
            .start_point_reader
            .all(Visibility::Only(ranked_orgs.iter().copied().collect()))
            .await?;
        let depot = select_depot(&start_points, &ranked_orgs, req.start_point_name.as_deref())
            .map(|p| p.coordinate)
            .ok_or_else(|| ServiceError::InvalidInput("no start point configured".into()))?;

        let station_points: Vec<&StartPoint> = start_points
            .iter()
            .filter(|p| p.replenishes().contains(&kind))
            .collect();
        let station_coordinates: Vec<Coordinate> =
            station_points.iter().map(|p| p.coordinate).collect();
        // A collection tour must unload before it returns to the depot.
        let return_replenished = kind == CapacityKind::CargoVolume;

        let capacity = capacity(
            kind,
            &transporter,
            trailer.as_ref(),
            req.transporter_capacity_used,
        )?;
        if return_replenished && station_points.is_empty() {
            return Err(RoutingError::NoStation { kind }.into());
        }
        let route = optimizer
            .optimize(RouteRequest {
                transporter: &transporter,
                kind,
                capacity,
                stops: &stops,
                depot,
                stations: &station_coordinates,
                return_replenished,
            })
            .await?;
        if !route.unserved.is_empty() {
            tracing::warn!(
                unserved = route.unserved.len(),
                "route leaves stops unserved"
            );
        }
        let station_visits: Vec<StationVisit> = route
            .refill_station_indices
            .iter()
            .filter_map(|&i| station_points.get(i))
            .map(|p| StationVisit {
                name: p.name.clone(),
                coordinate: p.coordinate,
            })
            .collect();
        Ok(ComputedRoute {
            total_demand: stops.iter().map(|s| s.demand).sum(),
            station_visits,
            route,
        })
    }

    /// Loads the plan, applies a state-transition closure, persists, and
    /// publishes the resulting events. Used by `start`, `cancel`, `fail`;
    /// `replace_details` doesn't fit because it emits no events.
    async fn transition<F>(
        &self,
        id: Id<OperationPlan>,
        f: F,
    ) -> Result<OperationPlan, ServiceError>
    where
        F: FnOnce(&mut OperationPlan) -> Result<Vec<DomainEvent>, OperationPlanError>,
    {
        let mut plan = self.reader.by_id(id).await?;
        ensure_module_active(
            self.activation.as_ref(),
            plan.module(),
            plan.organization_id(),
        )
        .await?;
        let events = f(&mut plan)?;
        self.writer.save(&plan).await?;
        self.event_bus.publish_all(events).await;
        Ok(plan)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plan.id = %id))]
    pub async fn delete(&self, id: Id<OperationPlan>) -> Result<(), ServiceError> {
        let plan = self.reader.by_id(id).await?;
        self.writer.delete(id).await?;
        // Comments have no cascading FK: the subject is polymorphic.
        let deleted_comments = self
            .comment_writer
            .delete_for_subject(CommentSubject::Plan(id))
            .await?;
        tracing::debug!(deleted_comments, "removed comments for deleted plan");
        self.event_bus
            .publish(DomainEvent::OperationPlanDeleted {
                plan_id: id,
                module: plan.module().clone(),
                stops: plan.stops().to_vec(),
            })
            .await;
        Ok(())
    }

    /// Called by a module in the same flow that deletes the object behind `stop`.
    /// Mirrors the cascade the irrigation tables used to have: finished plans lose
    /// the entry and its amount as well.
    #[tracing::instrument(level = "debug", skip_all, fields(%module, %stop))]
    pub async fn remove_stop(
        &self,
        module: &ModuleKey,
        stop: StopRef,
    ) -> Result<u64, ServiceError> {
        Ok(self.writer.remove_stop(module, stop).await?)
    }
}

fn capacity(
    kind: CapacityKind,
    transporter: &Vehicle,
    trailer: Option<&Vehicle>,
    capacity_used: bool,
) -> Result<Liters, ServiceError> {
    route_capacity(kind, transporter, trailer, capacity_used).map_err(|e| {
        ServiceError::VehicleCapacityMissing {
            vehicle: e.vehicle.value(),
            kind: e.kind.to_string(),
        }
    })
}

/// The organizations whose start points serve a plan of `org`: itself, then
/// its ancestors nearest first (a depot is configured once at the tenant and
/// inherited down), then its sub-units. Sibling tenants never appear, so a
/// plan can neither start from nor refill at another tenant's station.
fn related_orgs_nearest_first(
    hierarchy: &OrgHierarchy,
    org: Id<Organization>,
) -> Vec<Id<Organization>> {
    let mut ranked: Vec<Id<Organization>> = hierarchy.ancestors_from_root(org);
    ranked.reverse();
    if ranked.is_empty() {
        ranked.push(org);
    }
    ranked.extend(
        hierarchy
            .descendants_or_self(org)
            .into_iter()
            .filter(|id| *id != org),
    );
    ranked
}

/// A name match beats the default, and among candidates the one closest to
/// the plan's organization wins.
fn select_depot<'a>(
    start_points: &'a [StartPoint],
    ranked_orgs: &[Id<Organization>],
    name: Option<&str>,
) -> Option<&'a StartPoint> {
    let nearest = |pred: &dyn Fn(&StartPoint) -> bool| {
        ranked_orgs.iter().find_map(|org| {
            start_points
                .iter()
                .find(|p| p.organization_id() == *org && pred(p))
        })
    };
    name.and_then(|name| nearest(&|p| p.name.as_str() == name))
        .or_else(|| nearest(&|p| p.is_default()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::start_point::StartPointSnapshot;
    use std::collections::BTreeSet;

    fn org(n: u128) -> Id<Organization> {
        Id::new(uuid::Uuid::from_u128(n))
    }

    fn point(name: &str, org_id: Id<Organization>, is_default: bool) -> StartPoint {
        StartPoint::reconstitute(StartPointSnapshot {
            id: uuid::Uuid::now_v7(),
            name: name.to_string(),
            latitude: 54.0,
            longitude: 9.0,
            replenishes: BTreeSet::new(),
            is_default,
            organization_id: org_id.value(),
        })
    }

    /// root(1) → tenant(2) → unit(3); tenant(4) is a sibling tenant.
    fn hierarchy() -> OrgHierarchy {
        OrgHierarchy::from_pairs([
            (org(1), None),
            (org(2), Some(org(1))),
            (org(3), Some(org(2))),
            (org(4), Some(org(1))),
        ])
    }

    #[test]
    fn related_orgs_are_self_then_ancestors_then_sub_units_never_siblings() {
        let ranked = related_orgs_nearest_first(&hierarchy(), org(2));
        assert_eq!(ranked, vec![org(2), org(1), org(3)]);
    }

    #[test]
    fn unknown_org_still_ranks_itself() {
        assert_eq!(
            related_orgs_nearest_first(&hierarchy(), org(9)),
            vec![org(9)]
        );
    }

    #[test]
    fn own_default_beats_the_ancestors_default() {
        let points = vec![point("Root", org(1), true), point("Own", org(2), true)];
        let ranked = related_orgs_nearest_first(&hierarchy(), org(2));
        assert_eq!(
            select_depot(&points, &ranked, None).unwrap().name.as_str(),
            "Own"
        );
    }

    #[test]
    fn ancestor_default_is_inherited_when_nothing_closer_exists() {
        let points = vec![point("Root", org(1), true), point("Unit", org(3), false)];
        let ranked = related_orgs_nearest_first(&hierarchy(), org(2));
        assert_eq!(
            select_depot(&points, &ranked, None).unwrap().name.as_str(),
            "Root"
        );
    }

    #[test]
    fn name_match_beats_the_default_and_an_unknown_name_falls_back() {
        let points = vec![point("Root", org(1), true), point("Nord", org(2), false)];
        let ranked = related_orgs_nearest_first(&hierarchy(), org(2));
        assert_eq!(
            select_depot(&points, &ranked, Some("Nord"))
                .unwrap()
                .name
                .as_str(),
            "Nord"
        );
        assert_eq!(
            select_depot(&points, &ranked, Some("Nirgends"))
                .unwrap()
                .name
                .as_str(),
            "Root"
        );
    }

    #[test]
    fn a_sibling_tenants_default_is_never_chosen() {
        let points = vec![point("Fremd", org(4), true)];
        let ranked = related_orgs_nearest_first(&hierarchy(), org(2));
        assert!(select_depot(&points, &ranked, None).is_none());
    }
}
