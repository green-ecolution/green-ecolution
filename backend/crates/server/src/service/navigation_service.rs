use std::{collections::BTreeSet, sync::Arc};

use domain::{
    operation_plan::{ItineraryEntry, OperationPlan, OperationPlanStatus},
    planning::ResolvedStop,
    routing::{TurnByTurnRequest, TurnByTurnRoute, TurnByTurnRouter},
    shared::coordinates::Coordinate,
    vehicle::VehicleReader,
};

use super::ServiceError;

// Valhalla rejects requests above its configured location limit.
pub const MAX_NAVIGATION_WAYPOINTS: usize = 10;

pub struct NavigationService {
    router: Arc<dyn TurnByTurnRouter>,
    vehicles: Arc<dyn VehicleReader>,
}

impl NavigationService {
    pub fn new(router: Arc<dyn TurnByTurnRouter>, vehicles: Arc<dyn VehicleReader>) -> Self {
        Self { router, vehicles }
    }

    #[tracing::instrument(level = "debug", skip_all, fields(remaining = remaining.len()))]
    pub async fn route(
        &self,
        plan: &OperationPlan,
        stops: &[ResolvedStop],
        remaining: &[usize],
        origin: Coordinate,
        heading: Option<f64>,
    ) -> Result<TurnByTurnRoute, ServiceError> {
        if !matches!(
            plan.status(),
            OperationPlanStatus::Planned | OperationPlanStatus::Active
        ) {
            return Err(ServiceError::InvalidInput(
                "plan is no longer driven".into(),
            ));
        }
        let waypoints = waypoints(&positions(plan.itinerary(), stops), remaining)?;
        let transporter = plan
            .transporter_id()
            .ok_or_else(|| ServiceError::InvalidInput("plan has no transporter".into()))?;
        let vehicle = self.vehicles.by_id(transporter).await?.dimension;
        Ok(self
            .router
            .route(TurnByTurnRequest {
                origin,
                heading,
                waypoints,
                vehicle,
            })
            .await?)
    }
}

/// One itinerary entry reduced to what routing needs.
#[derive(Debug, Clone, Copy)]
pub struct EntryPosition {
    pub index: usize,
    pub coordinate: Option<Coordinate>,
}

pub fn positions(itinerary: &[ItineraryEntry], stops: &[ResolvedStop]) -> Vec<EntryPosition> {
    itinerary
        .iter()
        .enumerate()
        .map(|(index, entry)| EntryPosition {
            index,
            coordinate: match entry {
                ItineraryEntry::Depot(v) | ItineraryEntry::Station(v) => Some(v.coordinate),
                ItineraryEntry::Stop(stop) => stops
                    .iter()
                    .find(|s| s.stop == *stop)
                    .and_then(|s| s.location),
            },
        })
        .collect()
}

pub fn waypoints(
    entries: &[EntryPosition],
    remaining: &[usize],
) -> Result<Vec<Coordinate>, ServiceError> {
    if remaining.is_empty() {
        return Err(ServiceError::InvalidInput(
            "remaining must not be empty".into(),
        ));
    }
    let mut seen = BTreeSet::new();
    let mut picked = Vec::new();
    for &index in remaining {
        if !seen.insert(index) {
            return Err(ServiceError::InvalidInput(format!(
                "index {index} listed twice"
            )));
        }
        let entry = entries
            .get(index)
            .ok_or_else(|| ServiceError::InvalidInput(format!("index {index} out of range")))?;
        picked.extend(entry.coordinate);
    }
    picked.truncate(MAX_NAVIGATION_WAYPOINTS);
    if picked.is_empty() {
        return Err(ServiceError::InvalidInput(
            "no remaining entry has a position".into(),
        ));
    }
    Ok(picked)
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{operation_plan::StationVisit, planning::StopRef, start_point::StartPointName};

    fn entry(index: usize, lat: Option<f64>) -> EntryPosition {
        EntryPosition {
            index,
            coordinate: lat.map(|l| Coordinate::new(l, l).unwrap()),
        }
    }

    #[test]
    fn positions_take_depot_stations_and_resolved_stops() {
        let known = StopRef::new(uuid::Uuid::now_v7());
        let unknown = StopRef::new(uuid::Uuid::now_v7());
        let depot = StationVisit {
            name: StartPointName::new("Hof").unwrap(),
            coordinate: Coordinate::new(54.0, 9.0).unwrap(),
        };
        let itinerary = vec![
            ItineraryEntry::Depot(depot),
            ItineraryEntry::Stop(known),
            ItineraryEntry::Stop(unknown),
        ];
        let stops = vec![ResolvedStop {
            stop: known,
            label: "A".into(),
            location: Some(Coordinate::new(54.1, 9.1).unwrap()),
            demand: 1.0,
        }];

        let p = positions(&itinerary, &stops);

        assert_eq!(p.len(), 3);
        assert!(p[0].coordinate.is_some());
        assert_eq!(p[1].coordinate.map(|c| c.latitude()), Some(54.1));
        assert!(p[2].coordinate.is_none());
    }

    #[test]
    fn picks_coordinates_in_the_given_order() {
        let entries = vec![
            entry(0, Some(1.0)),
            entry(1, Some(2.0)),
            entry(2, Some(3.0)),
        ];
        let picked = waypoints(&entries, &[2, 1]).unwrap();
        assert_eq!(
            picked.iter().map(|c| c.latitude()).collect::<Vec<_>>(),
            vec![3.0, 2.0]
        );
    }

    #[test]
    fn routes_over_the_first_ten_positioned_entries_only() {
        let entries: Vec<_> = (0..15).map(|i| entry(i, Some(i as f64 + 1.0))).collect();
        let remaining: Vec<usize> = (0..15).collect();
        let picked = waypoints(&entries, &remaining).unwrap();
        assert_eq!(
            picked.iter().map(|c| c.latitude()).collect::<Vec<_>>(),
            (1..=10).map(f64::from).collect::<Vec<_>>()
        );
        let mut bad = remaining;
        bad.push(99);
        assert!(waypoints(&entries, &bad).is_err());
    }

    #[test]
    fn skips_entries_without_position() {
        let entries = vec![entry(0, None), entry(1, Some(2.0))];
        assert_eq!(waypoints(&entries, &[0, 1]).unwrap().len(), 1);
    }

    #[test]
    fn rejects_empty_duplicate_and_out_of_range_indices() {
        let entries = vec![entry(0, Some(1.0))];
        assert!(waypoints(&entries, &[]).is_err());
        assert!(waypoints(&entries, &[0, 0]).is_err());
        assert!(waypoints(&entries, &[1]).is_err());
    }

    #[test]
    fn nothing_left_with_a_position_is_an_error() {
        let entries = vec![entry(0, None)];
        assert!(waypoints(&entries, &[0]).is_err());
    }
}
