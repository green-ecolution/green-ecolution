use std::time::Duration;

use crate::config::ReadyProbe;

pub async fn check(probe: &ReadyProbe, client: &reqwest::Client) -> bool {
    match probe {
        ReadyProbe::Http(url) => client
            .get(url)
            .send()
            .await
            .is_ok_and(|response| response.status().is_success()),
        ReadyProbe::Tcp(addr) => tokio::net::TcpStream::connect(addr.as_str()).await.is_ok(),
    }
}

/// Polls until the probe succeeds; the caller races this against the process exiting.
pub async fn wait_ready(probe: &ReadyProbe, interval: Duration) {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap_or_default();
    while !check(probe, &client).await {
        tokio::time::sleep(interval).await;
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };

    use super::*;

    async fn serve_once(status: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 1024];
            let _ = socket.read(&mut buf).await;
            let response =
                format!("HTTP/1.1 {status}\r\ncontent-length: 0\r\nconnection: close\r\n\r\n");
            socket.write_all(response.as_bytes()).await.unwrap();
        });
        format!("http://{addr}/api/ready")
    }

    pub(crate) async fn closed_port() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);
        addr.to_string()
    }

    #[tokio::test]
    async fn http_probe_accepts_success_only() {
        let client = reqwest::Client::new();
        assert!(check(&ReadyProbe::Http(serve_once("200 OK").await), &client).await);
        assert!(
            !check(
                &ReadyProbe::Http(serve_once("503 Service Unavailable").await),
                &client
            )
            .await
        );
    }

    #[tokio::test]
    async fn tcp_probe_checks_for_a_listener() {
        let client = reqwest::Client::new();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let open = listener.local_addr().unwrap().to_string();

        assert!(check(&ReadyProbe::Tcp(open), &client).await);
        assert!(!check(&ReadyProbe::Tcp(closed_port().await), &client).await);
    }
}
