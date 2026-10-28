use domain::{
    operation_plan::ItineraryEntry,
    planning::{ResolvedStop, StopRef},
    shared::coordinates::Coordinate,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaypointKind {
    Depot,
    Stop,
    Station,
}

impl WaypointKind {
    fn as_str(self) -> &'static str {
        match self {
            WaypointKind::Depot => "depot",
            WaypointKind::Stop => "stop",
            WaypointKind::Station => "station",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Waypoint {
    pub name: String,
    pub coordinate: Coordinate,
    pub kind: WaypointKind,
}

/// The depot first and unnumbered, then every visit numbered in driving
/// order. Stops the itinerary does not know (a plan routed before the order
/// was persisted) follow in plan order, so no stop with a position is lost.
pub fn waypoints(itinerary: &[ItineraryEntry], stops: &[ResolvedStop]) -> Vec<Waypoint> {
    let mut result = Vec::new();
    let mut numbered: Vec<Waypoint> = Vec::new();
    let mut seen: Vec<StopRef> = Vec::new();
    for entry in itinerary {
        match entry {
            ItineraryEntry::Depot(depot) => {
                if result.is_empty() {
                    result.push(Waypoint {
                        name: depot.name.as_str().to_owned(),
                        coordinate: depot.coordinate,
                        kind: WaypointKind::Depot,
                    });
                }
            }
            ItineraryEntry::Station(station) => numbered.push(Waypoint {
                name: station.name.as_str().to_owned(),
                coordinate: station.coordinate,
                kind: WaypointKind::Station,
            }),
            ItineraryEntry::Stop(stop) => {
                seen.push(*stop);
                if let Some(waypoint) = stops
                    .iter()
                    .find(|s| s.stop == *stop)
                    .and_then(stop_waypoint)
                {
                    numbered.push(waypoint);
                }
            }
        }
    }
    numbered.extend(
        stops
            .iter()
            .filter(|s| !seen.contains(&s.stop))
            .filter_map(stop_waypoint),
    );
    for (i, mut waypoint) in numbered.into_iter().enumerate() {
        waypoint.name = format!("{}. {}", i + 1, waypoint.name);
        result.push(waypoint);
    }
    result
}

fn stop_waypoint(stop: &ResolvedStop) -> Option<Waypoint> {
    stop.location.map(|coordinate| Waypoint {
        name: stop.label.clone(),
        coordinate,
        kind: WaypointKind::Stop,
    })
}

/// Track name for a plan's route, in whatever language the plan itself uses.
///
/// The server must not invent prose here: a name it builds ships inside the
/// downloaded file, where no client-side translation can reach it. The plan's
/// own description is user-authored and therefore already in the right
/// language; without one, the date alone carries no language at all.
pub fn track_name(description: Option<&str>, date: &str) -> String {
    description
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .unwrap_or(date)
        .to_string()
}

pub fn render_gpx(name: &str, coords: &[Coordinate], waypoints: &[Waypoint]) -> String {
    let mut gpx = String::with_capacity(coords.len() * 64 + waypoints.len() * 96 + 256);
    gpx.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    gpx.push('\n');
    gpx.push_str(
        r#"<gpx version="1.1" creator="green-ecolution" xmlns="http://www.topografix.com/GPX/1/1">"#,
    );
    for w in waypoints {
        gpx.push_str(&format!(
            r#"<wpt lat="{}" lon="{}"><name>{}</name><type>{}</type></wpt>"#,
            w.coordinate.latitude(),
            w.coordinate.longitude(),
            xml_escape(&w.name),
            w.kind.as_str()
        ));
    }
    gpx.push_str("<trk><name>");
    gpx.push_str(&xml_escape(name));
    gpx.push_str("</name><trkseg>");
    for c in coords {
        gpx.push_str(&format!(
            r#"<trkpt lat="{}" lon="{}"/>"#,
            c.latitude(),
            c.longitude()
        ));
    }
    gpx.push_str("</trkseg></trk></gpx>");
    gpx
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{
        operation_plan::{ItineraryEntry, StationVisit},
        planning::{ResolvedStop, StopRef},
        start_point::StartPointName,
    };

    fn waypoint(name: &str, kind: WaypointKind) -> Waypoint {
        Waypoint {
            name: name.to_string(),
            coordinate: Coordinate::new(54.77, 9.45).unwrap(),
            kind,
        }
    }

    #[test]
    fn renders_waypoints_before_the_track() {
        let coords = vec![Coordinate::new(54.76, 9.43).unwrap()];
        let waypoints = vec![
            waypoint("Betriebshof", WaypointKind::Depot),
            waypoint("1. Stadtpark <Nord>", WaypointKind::Stop),
            waypoint("2. Klärwerk", WaypointKind::Station),
        ];
        let gpx = render_gpx("Route", &coords, &waypoints);
        let depot = gpx
            .find(r#"<wpt lat="54.77" lon="9.45"><name>Betriebshof</name><type>depot</type></wpt>"#)
            .unwrap();
        let stop = gpx
            .find("<name>1. Stadtpark &lt;Nord&gt;</name><type>stop</type>")
            .unwrap();
        let station = gpx
            .find("<name>2. Klärwerk</name><type>station</type>")
            .unwrap();
        let track = gpx.find("<trk>").unwrap();
        assert!(depot < stop && stop < station && station < track);
    }

    #[test]
    fn numbers_the_itinerary_and_names_the_depot_once() {
        let depot = place("Betriebshof", 54.76, 9.43);
        let station = place("Klärwerk", 54.80, 9.44);
        let stops = [stop_ref(1), stop_ref(2)];
        let itinerary = vec![
            ItineraryEntry::Depot(depot.clone()),
            ItineraryEntry::Station(station.clone()),
            ItineraryEntry::Stop(stops[1]),
            ItineraryEntry::Stop(stops[0]),
            ItineraryEntry::Depot(depot),
        ];
        let located = vec![
            located_stop(stops[0], "Stadtpark", 54.78, 9.41),
            located_stop(stops[1], "Hafen", 54.79, 9.42),
        ];
        let names: Vec<(String, WaypointKind)> = waypoints(&itinerary, &located)
            .into_iter()
            .map(|w| (w.name, w.kind))
            .collect();
        assert_eq!(
            names,
            vec![
                ("Betriebshof".to_string(), WaypointKind::Depot),
                ("1. Klärwerk".to_string(), WaypointKind::Station),
                ("2. Hafen".to_string(), WaypointKind::Stop),
                ("3. Stadtpark".to_string(), WaypointKind::Stop),
            ]
        );
    }

    #[test]
    fn stops_missing_from_the_itinerary_follow_in_plan_order() {
        let stops = [stop_ref(1), stop_ref(2), stop_ref(3)];
        let itinerary = vec![ItineraryEntry::Station(place("Klärwerk", 54.80, 9.44))];
        let located = vec![
            located_stop(stops[0], "Stadtpark", 54.78, 9.41),
            located_stop(stops[1], "Hafen", 54.79, 9.42),
            unlocated_stop(stops[2], "Ohne Position"),
        ];
        let names: Vec<String> = waypoints(&itinerary, &located)
            .into_iter()
            .map(|w| w.name)
            .collect();
        assert_eq!(names, vec!["1. Klärwerk", "2. Stadtpark", "3. Hafen"]);
    }

    fn place(name: &str, lat: f64, lon: f64) -> StationVisit {
        StationVisit {
            name: StartPointName::new(name.to_string()).unwrap(),
            coordinate: Coordinate::new(lat, lon).unwrap(),
        }
    }

    fn stop_ref(n: u128) -> StopRef {
        StopRef::new(uuid::Uuid::from_u128(n))
    }

    fn located_stop(stop: StopRef, label: &str, lat: f64, lon: f64) -> ResolvedStop {
        ResolvedStop {
            stop,
            label: label.to_string(),
            location: Some(Coordinate::new(lat, lon).unwrap()),
            demand: 0.0,
        }
    }

    fn unlocated_stop(stop: StopRef, label: &str) -> ResolvedStop {
        ResolvedStop {
            stop,
            label: label.to_string(),
            location: None,
            demand: 0.0,
        }
    }

    #[test]
    fn renders_track_points_and_escapes_name() {
        let coords = vec![
            Coordinate::new(54.76, 9.43).unwrap(),
            Coordinate::new(54.80, 9.44).unwrap(),
        ];
        let gpx = render_gpx("Route <Nord & Süd>", &coords, &[]);
        assert!(gpx.starts_with(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
        assert!(gpx.contains(r#"<trkpt lat="54.76" lon="9.43"/>"#));
        assert!(gpx.contains(r#"<trkpt lat="54.8" lon="9.44"/>"#));
        assert!(gpx.contains("<name>Route &lt;Nord &amp; Süd&gt;</name>"));
        assert!(gpx.ends_with("</trkseg></trk></gpx>"));
    }
}

#[cfg(test)]
mod track_name_tests {
    use super::*;

    #[test]
    fn a_plan_with_a_description_is_named_after_it() {
        assert_eq!(
            track_name(Some("Stadtpark Nord"), "2026-08-31"),
            "Stadtpark Nord"
        );
    }

    #[test]
    fn a_plan_without_a_description_falls_back_to_its_date() {
        assert_eq!(track_name(None, "2026-08-31"), "2026-08-31");
    }

    #[test]
    fn a_blank_description_counts_as_absent() {
        assert_eq!(track_name(Some("   "), "2026-08-31"), "2026-08-31");
    }
}
