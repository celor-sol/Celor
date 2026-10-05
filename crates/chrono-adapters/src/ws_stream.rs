use crate::provider::ProviderConnectionStatus;
use chrono_core::types::Slot;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time::sleep;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawSlotNotification {
    pub slot: Slot,
    pub parent: Option<Slot>,
    pub root: Option<Slot>,
}

/// Robust Solana WebSocket Stream with automatic reconnection and backoff.
pub struct SolanaWsStream {
    ws_url: String,
    status: Arc<RwLock<ProviderConnectionStatus>>,
    is_running: Arc<AtomicBool>,
}

impl SolanaWsStream {
    pub fn new(ws_url: impl Into<String>) -> Self {
        Self {
            ws_url: ws_url.into(),
            status: Arc::new(RwLock::new(ProviderConnectionStatus::Disconnected)),
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn get_status(&self) -> ProviderConnectionStatus {
        self.status.read().await.clone()
    }

    /// Spawns a background worker maintaining the WebSocket subscription.
    /// Emits parsed slot notifications to `out_sender`.
    pub fn start(&self, out_sender: mpsc::Sender<RawSlotNotification>) {
        if self.is_running.swap(true, Ordering::SeqCst) {
            return;
        }

        let ws_url = self.ws_url.clone();
        let status = self.status.clone();
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            let mut attempt = 0;

            while is_running.load(Ordering::SeqCst) {
                attempt += 1;
                *status.write().await = ProviderConnectionStatus::Reconnecting(attempt);

                info!("Connecting to Solana WebSocket: {} (attempt {})", ws_url, attempt);

                match connect_async(&ws_url).await {
                    Ok((ws_stream, _)) => {
                        *status.write().await = ProviderConnectionStatus::Connected;
                        attempt = 0;
                        info!("Connected to Solana WebSocket. Subscribing to slotSubscribe...");

                        let (mut write, mut read) = ws_stream.split();

                        let sub_msg = json!({
                            "jsonrpc": "2.0",
                            "id": 1,
                            "method": "slotSubscribe"
                        });

                        if let Err(e) = write.send(Message::Text(sub_msg.to_string())).await {
                            error!("Failed to send slotSubscribe: {}", e);
                            *status.write().await = ProviderConnectionStatus::Degraded(e.to_string());
                            sleep(Duration::from_millis(500)).await;
                            continue;
                        }

                        // Read loop
                        while let Some(msg_result) = read.next().await {
                            if !is_running.load(Ordering::SeqCst) {
                                break;
                            }

                            match msg_result {
                                Ok(Message::Text(text)) => {
                                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                                        if let Some(params) = val.get("params") {
                                            if let Some(result) = params.get("result") {
                                                if let Some(slot_u64) = result.get("slot").and_then(|s| s.as_u64()) {
                                                    let parent = result.get("parent").and_then(|p| p.as_u64()).map(Slot);
                                                    let root = result.get("root").and_then(|r| r.as_u64()).map(Slot);

                                                    let notification = RawSlotNotification {
                                                        slot: Slot(slot_u64),
                                                        parent,
                                                        root,
                                                    };

                                                    if out_sender.send(notification).await.is_err() {
                                                        // Receiver dropped
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                Ok(Message::Ping(p)) => {
                                    let _ = write.send(Message::Pong(p)).await;
                                }
                                Ok(Message::Close(_)) => {
                                    warn!("WebSocket server closed connection");
                                    break;
                                }
                                Err(e) => {
                                    warn!("WebSocket read error: {}", e);
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(e) => {
                        error!("WebSocket connection error: {}", e);
                        *status.write().await = ProviderConnectionStatus::Degraded(e.to_string());
                    }
                }

                // Exponential backoff before reconnecting: 500ms, 1s, 2s, capped at 5s
                let backoff_ms = (500 * (1 << attempt.min(3))).min(5000);
                sleep(Duration::from_millis(backoff_ms)).await;
            }

            *status.write().await = ProviderConnectionStatus::Disconnected;
        });
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }
}
