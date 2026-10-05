use chrono_adapters::cluster::ClusterConfig;
use serde::{Deserialize, Serialize};
use std::env;

/// Centralized configuration for Chrono Server.
///
/// Invariant: Default host is strictly 127.0.0.1 (local loopback),
/// never 0.0.0.0 unless explicitly configured.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cluster: String,
    pub rpc_url: String,
    pub ws_url: String,
    pub yellowstone_endpoint: Option<String>,
    pub yellowstone_token: Option<String>,
    pub source: String, // "rpc", "yellowstone", "local-geyser"
    pub event_buffer_capacity: usize,
    pub max_ws_clients: usize,
    pub log_level: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        let cluster_conf = ClusterConfig::testnet();
        Self {
            host: "127.0.0.1".to_string(),
            port: 8900,
            cluster: "testnet".to_string(),
            rpc_url: cluster_conf.rpc_url,
            ws_url: cluster_conf.ws_url,
            yellowstone_endpoint: None,
            yellowstone_token: None,
            source: "rpc".to_string(),
            event_buffer_capacity: 10_000,
            max_ws_clients: 100,
            log_level: "info".to_string(),
        }
    }
}

impl ServerConfig {
    /// Loads configuration from environment variables with sensible defaults.
    pub fn from_env() -> Self {
        let cluster = env::var("CHRONO_CLUSTER").unwrap_or_else(|_| "testnet".to_string());
        let default_cluster_conf = match cluster.to_lowercase().as_str() {
            "devnet" => ClusterConfig::devnet(),
            "mainnet" | "mainnet-beta" => ClusterConfig::mainnet_beta(),
            _ => ClusterConfig::testnet(),
        };

        let host = env::var("CHRONO_HTTP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env::var("CHRONO_HTTP_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8900);

        let rpc_url = env::var("CHRONO_RPC_URL").unwrap_or(default_cluster_conf.rpc_url);
        let ws_url = env::var("CHRONO_WS_URL").unwrap_or(default_cluster_conf.ws_url);

        let yellowstone_endpoint = env::var("CHRONO_YELLOWSTONE_ENDPOINT").ok();
        let yellowstone_token = env::var("CHRONO_YELLOWSTONE_TOKEN").ok();

        let source = env::var("CHRONO_SOURCE").unwrap_or_else(|_| {
            if yellowstone_endpoint.is_some() {
                "yellowstone".to_string()
            } else {
                "rpc".to_string()
            }
        });

        let event_buffer_capacity = env::var("CHRONO_EVENT_BUFFER")
            .ok()
            .and_then(|b| b.parse().ok())
            .unwrap_or(10_000);

        let max_ws_clients = env::var("CHRONO_MAX_WS_CLIENTS")
            .ok()
            .and_then(|m| m.parse().ok())
            .unwrap_or(100);

        let log_level = env::var("CHRONO_LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

        Self {
            host,
            port,
            cluster,
            rpc_url,
            ws_url,
            yellowstone_endpoint,
            yellowstone_token,
            source,
            event_buffer_capacity,
            max_ws_clients,
            log_level,
        }
    }
}
