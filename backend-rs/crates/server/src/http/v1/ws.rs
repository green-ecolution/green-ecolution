use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::Response;
use serde::Deserialize;
use tokio::time::{MissedTickBehavior, interval};

use domain::info::RuntimeStatsProvider;

use crate::http::AppState;
use crate::http::v1::dto::runtime_stats::RuntimeStatsResponse;
use crate::service::AuthError;

#[derive(Deserialize)]
pub struct WsStatsQuery {
    token: Option<String>,
}

#[tracing::instrument(level = "info", skip_all)]
pub async fn ws_stats(
    ws: WebSocketUpgrade,
    Query(query): Query<WsStatsQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Response, AuthError> {
    if state.token_validator.is_enforced() {
        let token = query.token.ok_or(AuthError::MissingToken)?;
        state.token_validator.validate(&token).await?;
    }

    let provider = state.runtime_stats_provider.clone();
    let push_interval = state.runtime_stats_push_interval;
    Ok(ws.on_upgrade(move |socket| ws_stats_loop(socket, provider, push_interval)))
}

async fn ws_stats_loop(
    mut socket: WebSocket,
    provider: Arc<dyn RuntimeStatsProvider>,
    push_interval: Duration,
) {
    let mut ticker = interval(push_interval);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let stats = provider.snapshot().await;
                let payload = match serde_json::to_string(&RuntimeStatsResponse::from(&stats)) {
                    Ok(p) => p,
                    Err(error) => {
                        tracing::error!(%error, "failed to serialize runtime stats");
                        return;
                    }
                };
                if socket.send(Message::Text(payload.into())).await.is_err() {
                    return;
                }
            }
            msg = socket.recv() => match msg {
                None | Some(Ok(Message::Close(_))) | Some(Err(_)) => return,
                Some(Ok(_)) => {}
            }
        }
    }
}
