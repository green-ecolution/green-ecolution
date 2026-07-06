use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use serde::Deserialize;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::{MissedTickBehavior, interval};

const GITHUB_API_BASE: &str = "https://api.github.com";

#[derive(Deserialize)]
struct GhRelease {
    tag_name: String,
}

pub struct UpdateChecker {
    current: String,
    latest: RwLock<String>,
    repo: Option<String>,
}

impl UpdateChecker {
    pub fn new(current: String, repo: Option<String>) -> Self {
        let latest = RwLock::new(current.clone());
        Self {
            current,
            latest,
            repo,
        }
    }

    pub fn current(&self) -> &str {
        &self.current
    }

    pub async fn latest(&self) -> String {
        self.latest.read().await.clone()
    }

    pub async fn update_available(&self) -> bool {
        let latest = self.latest.read().await;
        normalize(&latest) != normalize(&self.current)
    }

    pub async fn refresh(&self, client: &Client) -> Result<(), reqwest::Error> {
        self.refresh_with_base(client, GITHUB_API_BASE).await
    }

    pub async fn refresh_with_base(
        &self,
        client: &Client,
        api_base: &str,
    ) -> Result<(), reqwest::Error> {
        let Some(repo) = &self.repo else {
            return Ok(());
        };
        let base = api_base.trim_end_matches('/');
        let url = format!("{base}/repos/{repo}/releases/latest");
        let release: GhRelease = client
            .get(&url)
            .header("User-Agent", "green-ecolution")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        let normalized = release.tag_name.trim_start_matches('v').to_string();
        *self.latest.write().await = normalized;
        Ok(())
    }
}

fn normalize(version: &str) -> &str {
    version.trim_start_matches('v')
}

pub fn spawn(
    checker: Arc<UpdateChecker>,
    client: Client,
    refresh_interval: Duration,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = interval(refresh_interval);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            ticker.tick().await;
            if let Err(error) = checker.refresh(&client).await {
                tracing::warn!(%error, "update check failed; keeping previous latest");
            }
        }
    })
}
