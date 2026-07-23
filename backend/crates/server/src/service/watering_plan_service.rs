use std::sync::Arc;

use domain::{
    Id,
    cluster::{TreeCluster, TreeClusterReader},
    events::DomainEvent,
    organization::{Organization, OrganizationReader},
    routing::{OptimizedRoute, RouteOptimizer, RouteStop},
    shared::pagination::{Page, Pagination},
    start_point::{StartPoint, StartPointReader},
    vehicle::{Vehicle, VehicleReader},
    watering_plan::{
        RefillPoint, RouteMetrics, WateringPlan, WateringPlanDraft, WateringPlanError,
        WateringPlanEvaluation, WateringPlanReader, WateringPlanSearchQuery, WateringPlanUpdate,
        WateringPlanView, WateringPlanWriter,
    },
};

use super::{ServiceError, event_bus::EventBus};

pub struct WateringPlanService {
    reader: Arc<dyn WateringPlanReader>,
    writer: Arc<dyn WateringPlanWriter>,
    cluster_reader: Arc<dyn TreeClusterReader>,
    vehicle_reader: Arc<dyn VehicleReader>,
    event_bus: Arc<dyn EventBus>,
    route_optimizer: Option<Arc<dyn RouteOptimizer>>,
    tree_demand_liters: f64,
    start_point_reader: Arc<dyn StartPointReader>,
    org_reader: Arc<dyn OrganizationReader>,
}

/// Result of a route computation: the optimized route plus the summed
/// cluster water demand (which the solver does not report back).
pub struct ComputedRoute {
    pub route: OptimizedRoute,
    pub total_water_liters: f64,
    pub refill_points: Vec<RefillPoint>,
}

impl WateringPlanService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        reader: Arc<dyn WateringPlanReader>,
        writer: Arc<dyn WateringPlanWriter>,
        cluster_reader: Arc<dyn TreeClusterReader>,
        vehicle_reader: Arc<dyn VehicleReader>,
        event_bus: Arc<dyn EventBus>,
        route_optimizer: Option<Arc<dyn RouteOptimizer>>,
        tree_demand_liters: f64,
        start_point_reader: Arc<dyn StartPointReader>,
        org_reader: Arc<dyn OrganizationReader>,
    ) -> Self {
        Self {
            reader,
            writer,
            cluster_reader,
            vehicle_reader,
            event_bus,
            route_optimizer,
            tree_demand_liters,
            start_point_reader,
            org_reader,
        }
    }

    /// Every referenced cluster must be reachable for the plan's org: owned
    /// inside its subtree (shares extend this in the sharing feature).
    async fn ensure_clusters_accessible(
        &self,
        cluster_ids: &[Id<TreeCluster>],
        plan_org: Id<Organization>,
    ) -> Result<(), ServiceError> {
        if cluster_ids.is_empty() {
            return Ok(());
        }
        let hierarchy = self.org_reader.hierarchy().await?;
        let clusters = self.cluster_reader.by_ids(cluster_ids).await?;
        if clusters
            .iter()
            .all(|c| hierarchy.is_descendant_or_self(c.organization_id(), plan_org))
        {
            Ok(())
        } else {
            Err(ServiceError::OrganizationMismatch)
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn search_view(
        &self,
        query: WateringPlanSearchQuery,
        pagination: Pagination,
    ) -> Result<Page<WateringPlanView>, ServiceError> {
        Ok(self.reader.view_search(query, pagination).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plan.id = %id))]
    pub async fn by_id(&self, id: Id<WateringPlan>) -> Result<WateringPlan, ServiceError> {
        Ok(self.reader.by_id(id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plan.id = %id))]
    pub async fn view_by_id(&self, id: Id<WateringPlan>) -> Result<WateringPlanView, ServiceError> {
        Ok(self.reader.view_by_id(id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plan.id = %id))]
    pub async fn evaluations(
        &self,
        id: Id<WateringPlan>,
    ) -> Result<Vec<WateringPlanEvaluation>, ServiceError> {
        Ok(self.reader.evaluations(id).await?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn create(&self, draft: WateringPlanDraft) -> Result<WateringPlan, ServiceError> {
        self.ensure_clusters_accessible(&draft.cluster_ids, draft.organization_id)
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
        id: Id<WateringPlan>,
        update: WateringPlanUpdate,
    ) -> Result<WateringPlan, ServiceError> {
        let mut plan = self.reader.by_id(id).await?;
        self.ensure_clusters_accessible(&update.cluster_ids, plan.organization_id())
            .await?;
        plan.replace_details(update).map_err(map_plan_error)?;
        if self.route_optimizer.is_some() {
            // Edited cluster/vehicle set invalidates the old route; a failed
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
    pub async fn start(&self, id: Id<WateringPlan>) -> Result<WateringPlan, ServiceError> {
        self.transition(id, |plan| plan.start()).await
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plan.id = %id))]
    pub async fn revert_start(&self, id: Id<WateringPlan>) -> Result<WateringPlan, ServiceError> {
        self.transition(id, |plan| plan.revert_start()).await
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plan.id = %id))]
    pub async fn cancel(
        &self,
        id: Id<WateringPlan>,
        note: String,
    ) -> Result<WateringPlan, ServiceError> {
        self.transition(id, move |plan| plan.cancel(note)).await
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plan.id = %id))]
    pub async fn fail(
        &self,
        id: Id<WateringPlan>,
        note: String,
    ) -> Result<WateringPlan, ServiceError> {
        self.transition(id, move |plan| plan.fail(note)).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn preview_route(
        &self,
        cluster_ids: Vec<Id<TreeCluster>>,
        transporter_id: Id<Vehicle>,
        trailer_id: Option<Id<Vehicle>>,
        start_point_name: Option<String>,
    ) -> Result<ComputedRoute, ServiceError> {
        self.compute_route(&cluster_ids, transporter_id, trailer_id, start_point_name)
            .await
    }

    /// Route failures must never block plan persistence: the plan is already
    /// saved and the frontend surfaces a missing route via `distance == 0`.
    async fn apply_route(&self, plan: &mut WateringPlan) {
        let Some(transporter_id) = plan.transporter_id() else {
            return;
        };
        match self
            .compute_route(
                plan.cluster_ids(),
                transporter_id,
                plan.trailer_id(),
                plan.start_point_name.clone(),
            )
            .await
        {
            Ok(computed) => {
                plan.set_metrics(RouteMetrics {
                    distance: Some(computed.route.distance),
                    total_water_required: Some(computed.total_water_liters),
                    refill_count: computed.route.refill_count,
                    duration: computed.route.duration,
                    route_geometry: Some(computed.route.geometry),
                    refill_points: computed.refill_points,
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

    async fn compute_route(
        &self,
        cluster_ids: &[Id<TreeCluster>],
        transporter_id: Id<Vehicle>,
        trailer_id: Option<Id<Vehicle>>,
        start_point_name: Option<String>,
    ) -> Result<ComputedRoute, ServiceError> {
        let optimizer = self
            .route_optimizer
            .as_ref()
            .ok_or(ServiceError::FeatureDisabled { feature: "routing" })?;
        let transporter = self.vehicle_reader.by_id(transporter_id).await?;
        let trailer = match trailer_id {
            Some(id) => Some(self.vehicle_reader.by_id(id).await?),
            None => None,
        };
        let clusters = self.cluster_reader.by_ids(cluster_ids).await?;
        let stops: Vec<RouteStop> = clusters
            .iter()
            .filter_map(|cluster| {
                cluster.coordinates().map(|location| RouteStop {
                    cluster_id: cluster.id,
                    location,
                    demand_liters: cluster.tree_ids.len() as f64 * self.tree_demand_liters,
                })
            })
            .collect();
        if stops.is_empty() {
            return Err(ServiceError::InvalidInput(
                "no cluster with coordinates to route".into(),
            ));
        }
        let start_points = self.start_point_reader.all().await?;
        let depot = match start_point_name.as_deref() {
            Some(name) => start_points
                .iter()
                .find(|p| p.name.as_str() == name)
                .or_else(|| start_points.iter().find(|p| p.is_default())),
            None => start_points.iter().find(|p| p.is_default()),
        }
        .map(|p| p.coordinate)
        .ok_or_else(|| ServiceError::InvalidInput("no start point configured".into()))?;

        let refill_start_points: Vec<&StartPoint> =
            start_points.iter().filter(|p| p.watering_point()).collect();
        let refill_stations: Vec<domain::shared::coordinates::Coordinate> =
            refill_start_points.iter().map(|p| p.coordinate).collect();

        let route = optimizer
            .optimize(
                &transporter,
                trailer.as_ref(),
                &stops,
                depot,
                &refill_stations,
            )
            .await?;
        if !route.unserved.is_empty() {
            tracing::warn!(
                unserved = route.unserved.len(),
                "route leaves clusters unserved"
            );
        }
        let refill_points: Vec<RefillPoint> = route
            .refill_station_indices
            .iter()
            .filter_map(|&i| refill_start_points.get(i))
            .map(|p| RefillPoint {
                name: p.name.clone(),
                coordinate: p.coordinate,
            })
            .collect();
        let total_water_liters = stops.iter().map(|s| s.demand_liters).sum();
        Ok(ComputedRoute {
            route,
            total_water_liters,
            refill_points,
        })
    }

    /// Loads the plan, applies a state-transition closure, persists, and
    /// publishes the resulting events. Used by `start`, `cancel`, `fail`;
    /// `replace_details` doesn't fit because it emits no events.
    async fn transition<F>(&self, id: Id<WateringPlan>, f: F) -> Result<WateringPlan, ServiceError>
    where
        F: FnOnce(&mut WateringPlan) -> Result<Vec<DomainEvent>, WateringPlanError>,
    {
        let mut plan = self.reader.by_id(id).await?;
        let events = f(&mut plan).map_err(map_plan_error)?;
        self.writer.save(&plan).await?;
        self.event_bus.publish_all(events).await;
        Ok(plan)
    }

    #[tracing::instrument(level = "debug", skip_all, fields(plan.id = %id))]
    pub async fn delete(&self, id: Id<WateringPlan>) -> Result<(), ServiceError> {
        let plan = self.reader.by_id(id).await?;
        let cluster_ids = plan.cluster_ids().to_vec();
        self.writer.delete(id).await?;
        self.event_bus
            .publish(DomainEvent::WateringPlanDeleted {
                plan_id: id,
                cluster_ids,
            })
            .await;
        Ok(())
    }
}

fn map_plan_error(e: WateringPlanError) -> ServiceError {
    ServiceError::InvalidInput(e.to_string())
}
