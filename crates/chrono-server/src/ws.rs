use crate::api::AppState;
use crate::envelope::ChronoServiceEvent;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;
use std::time::Duration;
use tracing::{info, warn};

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "hello")]
    Hello {
        schema_version: Option<u32>,
        client_id: Option<String>,
        last_sequence: Option<u64>,
    },
    #[serde(rename = "pong")]
    Pong,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
#[allow(clippy::large_enum_variant)]
pub enum ServerMessage {
    #[serde(rename = "welcome")]
    Welcome {
        server_version: String,
        current_sequence: u64,
        snapshot_sequence: u64,
    },
    #[serde(rename = "gap")]
    Gap {
        requested_sequence: u64,
        oldest_available_sequence: u64,
        message: String,
    },
    #[serde(rename = "event")]
    Event { event: ChronoServiceEvent },
    #[serde(rename = "ping")]
    Ping { timestamp_ms: u64 },
}

pub fn ws_routes(
    state: AppState,
    broadcast_rx_provider: tokio::sync::broadcast::Sender<ChronoServiceEvent>,
) -> Router {
    Router::new()
        .route(
            "/api/v1/stream",
            get(move |ws: WebSocketUpgrade| {
                let rx_prov = broadcast_rx_provider.clone();
                async move {
                    ws.on_upgrade(move |socket| handle_socket(socket, state, rx_prov))
                }
            }),
        )
}

async fn handle_socket(
    socket: WebSocket,
    state: AppState,
    broadcast_tx: tokio::sync::broadcast::Sender<ChronoServiceEvent>,
) {
    state.ws_clients.fetch_add(1, Ordering::Relaxed);
    let (mut sender, mut receiver) = socket.split();
    let mut broadcast_rx = broadcast_tx.subscribe();

    info!("New WebSocket client connected to /api/v1/stream");

    // Send initial Welcome message
    let (current_seq, snapshot_seq) = {
        let ws_count = state.ws_clients.load(Ordering::Relaxed) as usize;
        let snap = state.engine.read().await.build_snapshot(ws_count);
        (snap.sequence, snap.sequence)
    };

    let welcome = ServerMessage::Welcome {
        server_version: "0.1.0".to_string(),
        current_sequence: current_seq,
        snapshot_sequence: snapshot_seq,
    };

    if let Ok(welcome_json) = serde_json::to_string(&welcome) {
        let _ = sender.send(Message::Text(welcome_json)).await;
    }

    // Ping interval timer (every 15 seconds)
    let mut ping_interval = tokio::time::interval(Duration::from_secs(15));
    // Skip the immediate first tick
    ping_interval.tick().await;

    loop {
        tokio::select! {
            // Outbound live events from Chrono Core broadcast channel
            event_res = broadcast_rx.recv() => {
                match event_res {
                    Ok(event) => {
                        let msg = ServerMessage::Event { event };
                        if let Ok(json) = serde_json::to_string(&msg) {
                            if sender.send(Message::Text(json)).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(dropped)) => {
                        warn!("WebSocket client lagged, dropped {} events", dropped);
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }

            // Inbound messages from Client (Hello handshake / Replay request / Pong)
            client_msg = receiver.next() => {
                match client_msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(parsed) = serde_json::from_str::<ClientMessage>(&text) {
                            match parsed {
                                ClientMessage::Hello { last_sequence, .. } => {
                                    if let Some(last_seq) = last_sequence {
                                        let replay_res = state.engine.read().await.get_events_since(last_seq);
                                        match replay_res {
                                            Ok(replayed_events) => {
                                                for rev in replayed_events {
                                                    let msg = ServerMessage::Event { event: rev };
                                                    if let Ok(json) = serde_json::to_string(&msg) {
                                                        if sender.send(Message::Text(json)).await.is_err() {
                                                            break;
                                                        }
                                                    }
                                                }
                                            }
                                            Err(oldest) => {
                                                let gap_msg = ServerMessage::Gap {
                                                    requested_sequence: last_seq,
                                                    oldest_available_sequence: oldest,
                                                    message: "Requested sequence is older than retained event buffer. Please fetch /api/v1/snapshot".to_string(),
                                                };
                                                if let Ok(json) = serde_json::to_string(&gap_msg) {
                                                    let _ = sender.send(Message::Text(json)).await;
                                                }
                                            }
                                        }
                                    }
                                }
                                ClientMessage::Pong => {}
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        break;
                    }
                    _ => {}
                }
            }

            // Heartbeat Ping
            _ = ping_interval.tick() => {
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                let ping = ServerMessage::Ping { timestamp_ms: now_ms };
                if let Ok(json) = serde_json::to_string(&ping) {
                    if sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
        }
    }

    state.ws_clients.fetch_sub(1, Ordering::Relaxed);
    info!("WebSocket client disconnected");
}
