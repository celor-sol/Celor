use async_trait::async_trait;
use chrono_core::types::{ProviderId, Slot};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("WebSocket error: {0}")]
    WebSocket(String),
    #[error("JSON RPC error: {0}")]
    JsonRpc(String),
    #[error("Connection closed: {0}")]
    ConnectionClosed(String),
    #[error("Invalid payload format: {0}")]
    InvalidPayload(String),
    #[error("Timeout: {0}")]
    Timeout(String),
}

/// Provider connection health lifecycle status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderConnectionStatus {
    Connected,
    Degraded(String),
    Reconnecting(u32),
    Disconnected,
}

impl fmt::Display for ProviderConnectionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connected => write!(f, "CONNECTED"),
            Self::Degraded(reason) => write!(f, "DEGRADED: {}", reason),
            Self::Reconnecting(attempt) => write!(f, "RECONNECTING (attempt {})", attempt),
            Self::Disconnected => write!(f, "DISCONNECTED"),
        }
    }
}

/// Capabilities explicitly exposed by this provider stream.
///
/// RULE: Do NOT hide provider limitations. If a free WebSocket provides slot
/// updates but NOT validator-local bank_id or UpdateParent, report them as false!
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    pub slot_subscription: bool,
    pub block_subscription: bool,
    /// Validator-local bank_id stream. False on free public RPC/WebSocket.
    pub bank_id_stream: bool,
    /// Fast leader handover UpdateParent marker stream. False on free public RPC.
    pub update_parent_stream: bool,
    /// BLS finalization certificate stream.
    pub bls_certificates: bool,
}

impl ProviderCapabilities {
    /// Capabilities for standard public Solana JSON-RPC / WebSocket.
    pub fn standard_public_rpc() -> Self {
        Self {
            slot_subscription: true,
            block_subscription: true,
            bank_id_stream: false,         // Explicit limitation: not exposed on standard public WS
            update_parent_stream: false,   // Explicit limitation: requires Agave 4.3 Geyser / shred stream
            bls_certificates: false,       // Explicit limitation: requires BlockFooter Geyser
        }
    }

    /// Capabilities for dedicated Yellowstone gRPC / Geyser provider (Phase 3).
    pub fn yellowstone_geyser() -> Self {
        Self {
            slot_subscription: true,
            block_subscription: true,
            bank_id_stream: true,
            update_parent_stream: true,
            bls_certificates: true,
        }
    }
}

/// Universal Provider Adapter interface.
#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    fn provider_id(&self) -> &ProviderId;
    fn status(&self) -> ProviderConnectionStatus;
    fn capabilities(&self) -> ProviderCapabilities;
    async fn connect(&mut self) -> Result<(), AdapterError>;
    async fn disconnect(&mut self) -> Result<(), AdapterError>;
    async fn fetch_current_slot(&self) -> Result<Slot, AdapterError>;
}
