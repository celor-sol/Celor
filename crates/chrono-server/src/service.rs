use crate::api::{api_routes, AppState};
use crate::config::ServerConfig;
use crate::envelope::ChronoServiceEvent;
use crate::state::CoreStateEngine;
use crate::ws::ws_routes;
use axum::Router;
use chrono_adapters::rpc_client::SolanaRpcClient;
use std::net::SocketAddr;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{broadcast, mpsc, RwLock};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

pub struct ChronoServer {
    config: ServerConfig,
    engine: Arc<RwLock<CoreStateEngine>>,
    broadcast_tx: broadcast::Sender<ChronoServiceEvent>,
    ws_clients: Arc<AtomicU64>,
    source_manager: Arc<crate::source::SourceManager>,
    event_rx: Arc<tokio::sync::Mutex<Option<mpsc::Receiver<chrono_core::events::ChronoEvent>>>>,
}

impl ChronoServer {
    pub fn new(config: ServerConfig) -> Self {
        let (broadcast_tx, _) = broadcast::channel(5_000);
        let engine = Arc::new(RwLock::new(CoreStateEngine::new(config.clone())));
        let ws_clients = Arc::new(AtomicU64::new(0));
        let (event_tx, event_rx) = mpsc::channel(1_000);
        let source_manager = Arc::new(crate::source::SourceManager::new(event_tx));

        Self {
            config,
            engine,
            broadcast_tx,
            ws_clients,
            source_manager,
            event_rx: Arc::new(tokio::sync::Mutex::new(Some(event_rx))),
        }
    }

    pub fn engine(&self) -> Arc<RwLock<CoreStateEngine>> {
        self.engine.clone()
    }

    pub fn broadcast_sender(&self) -> broadcast::Sender<ChronoServiceEvent> {
        self.broadcast_tx.clone()
    }

    /// Builds the combined Axum application router with CORS and routes.
    pub fn router(&self) -> Router {
        let rpc_client = if !self.config.rpc_url.is_empty() {
            Arc::new(RwLock::new(Some(Arc::new(SolanaRpcClient::new(self.config.rpc_url.clone())))))
        } else {
            Arc::new(RwLock::new(None))
        };

        let app_state = AppState {
            engine: self.engine.clone(),
            rpc_client,
            source_manager: Some(self.source_manager.clone()),
            start_time: Instant::now(),
            config: self.config.clone(),
            ws_clients: self.ws_clients.clone(),
        };

        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        Router::new()
            .merge(api_routes(app_state.clone()))
            .merge(ws_routes(app_state, self.broadcast_tx.clone()))
            .layer(cors)
    }

    /// Starts the Chrono server, source ingestion loop, and HTTP/WS server.
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr_str = format!("{}:{}", self.config.host, self.config.port);
        let addr: SocketAddr = addr_str.parse().map_err(|e| {
            format!("Invalid bind address '{}': {}", addr_str, e)
        })?;

        info!("============================================================");
        info!("CHRONO RUST SERVICE — STARTING");
        info!("============================================================");
        info!("Bind Address:        http://{}", addr);
        info!("Cluster:             {}", self.config.cluster);
        info!("Source:              {}", self.config.source);
        info!("RPC URL:             {}", self.config.rpc_url);
        info!("Event Buffer:        {} entries", self.config.event_buffer_capacity);
        info!("WebSocket Stream:    ws://{}/api/v1/stream", addr);
        info!("Snapshot Endpoint:   http://{}/api/v1/snapshot", addr);
        info!("============================================================");

        // 1. Start initial telemetry source
        self.source_manager
            .switch_source(
                &self.config.cluster,
                &self.config.source,
                &self.config.rpc_url,
                &self.config.ws_url,
            )
            .await
            .map_err(|e| format!("Failed to start initial source: {}", e))?;

        // 2. Start Source -> Core processing pipeline
        let mut event_rx = self
            .event_rx
            .lock()
            .await
            .take()
            .ok_or("event_rx already consumed")?;
        let engine_clone = self.engine.clone();
        let broadcast_tx_clone = self.broadcast_tx.clone();

        tokio::spawn(async move {
            while let Some(core_event) = event_rx.recv().await {
                let service_event = {
                    let mut eng = engine_clone.write().await;
                    eng.process_event(core_event)
                };

                // Broadcast to all active WebSocket clients
                let _ = broadcast_tx_clone.send(service_event);
            }
        });

        // 3. Start Axum HTTP & WebSocket Server
        let app = self.router();
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        info!("Chrono Service listening on http://{}", addr);

        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        info!("Chrono Service shutdown cleanly.");
        Ok(())
    }
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
    info!("Shutdown signal received, shutting down gracefully...");
}
