use serde::{Deserialize, Serialize};
use std::fmt;

/// Explicit capability state for protocol and streaming features.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityState {
    /// Actively evidenced on the cluster.
    Available,
    /// Verified not available or rejected by cluster.
    Unavailable,
    /// Unverified or uncertain under current free infrastructure.
    Unknown,
}

impl fmt::Display for CapabilityState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Available => write!(f, "AVAILABLE"),
            Self::Unavailable => write!(f, "UNAVAILABLE"),
            Self::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

/// Set of all consensus and streaming capabilities tracked by CHRONO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilitySet {
    pub alpenglow_votor: CapabilityState,
    pub genesis_certificate: CapabilityState,
    pub bank_aware_stream: CapabilityState,
    pub update_parent: CapabilityState,
    pub block_footer: CapabilityState,
    pub finality_evidence: CapabilityState,
}

impl Default for CapabilitySet {
    fn default() -> Self {
        Self {
            alpenglow_votor: CapabilityState::Unknown,
            genesis_certificate: CapabilityState::Unknown,
            bank_aware_stream: CapabilityState::Unknown,
            update_parent: CapabilityState::Unknown,
            block_footer: CapabilityState::Unknown,
            finality_evidence: CapabilityState::Unknown,
        }
    }
}
