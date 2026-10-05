use serde::{Deserialize, Serialize};

/// Cluster endpoint configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    pub name: String,
    pub rpc_url: String,
    pub ws_url: String,
}

impl ClusterConfig {
    /// Solana Testnet (prioritized for Alpenglow / Agave 4.4 beta validation).
    pub fn testnet() -> Self {
        Self {
            name: "testnet".to_string(),
            rpc_url: "https://api.testnet.solana.com".to_string(),
            ws_url: "wss://api.testnet.solana.com".to_string(),
        }
    }

    /// Solana Devnet (fallback).
    pub fn devnet() -> Self {
        Self {
            name: "devnet".to_string(),
            rpc_url: "https://api.devnet.solana.com".to_string(),
            ws_url: "wss://api.devnet.solana.com".to_string(),
        }
    }

    /// Solana Mainnet-Beta.
    pub fn mainnet_beta() -> Self {
        Self {
            name: "mainnet-beta".to_string(),
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            ws_url: "wss://api.mainnet-beta.solana.com".to_string(),
        }
    }

    /// Custom or local validator endpoint (`solana-test-validator`).
    pub fn custom(name: impl Into<String>, rpc_url: impl Into<String>, ws_url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            rpc_url: rpc_url.into(),
            ws_url: ws_url.into(),
        }
    }
}
