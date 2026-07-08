use domain::shared::coordinates::Coordinate;

pub fn render_gpx(name: &str, coords: &[Coordinate]) -> String {
    let mut gpx = String::with_capacity(coords.len() * 64 + 256);
    gpx.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    gpx.push('\n');
    gpx.push_str(
        r#"<gpx version="1.1" creator="green-ecolution" xmlns="http://www.topografix.com/GPX/1/1">"#,
    );
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

    #[test]
    fn renders_track_points_and_escapes_name() {
        let coords = vec![
            Coordinate::new(54.76, 9.43).unwrap(),
            Coordinate::new(54.80, 9.44).unwrap(),
        ];
        let gpx = render_gpx("Route <Nord & Süd>", &coords);
        assert!(gpx.starts_with(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
        assert!(gpx.contains(r#"<trkpt lat="54.76" lon="9.43"/>"#));
        assert!(gpx.contains(r#"<trkpt lat="54.8" lon="9.44"/>"#));
        assert!(gpx.contains("<name>Route &lt;Nord &amp; Süd&gt;</name>"));
        assert!(gpx.ends_with("</trkseg></trk></gpx>"));
    }
}
