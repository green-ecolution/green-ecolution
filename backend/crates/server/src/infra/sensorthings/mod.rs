//! HTTP client side of the SensorThings export. PR 1 only probes a target;
//! the sink that writes entities and observations builds on `auth`.

mod auth;
mod body;
mod probe;
mod sink;

pub use probe::SensorThingsProbe;
pub use sink::SensorThingsHttpSink;

use std::time::Duration;

/// Redirects are refused: the URL policy checked the configured host, not
/// wherever that host might send the request next.
pub fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("reqwest client must build")
}
