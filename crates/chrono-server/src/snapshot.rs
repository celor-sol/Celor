use crate::envelope::{ChronoServiceEvent, EventProvenance};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateBankSummary {
    pub bank_id: String,
    pub raw_bank_id: Option<u64>,
    pub slot: u64,
    pub parent_bank_id: Option<String>,
    pub blockhash: Option<String>,
    pub bank_hash: Option<String>,
    pub state: String, // "OBSERVED", "CANONICAL", "ABANDONED", "DEAD", "ROOTED"
    pub tx_count: u64,
    pub observed_at_ms: u64,
    pub provenance: EventProvenance,
    pub abandonment_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateParentSummary {
    pub slot: u64,
    pub cleared_bank_id: Option<u64>,
    pub replacement_bank_id: Option<u64>,
    pub parent_slot: u64,
    pub parent_block_id: Option<String>,
    pub reason: String,
    pub observed_at_ms: u64,
    pub provenance: EventProvenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotSnapshot {
    pub current_slot: u64,
    pub target_duration_ms: u64,
    pub elapsed_ms: u64,
    pub phase_ratio: f64,
    pub provenance: EventProvenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderLookaheadEntry {
    pub slot: u64,
    pub leader: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderSnapshot {
    pub current_leader: Option<String>,
    pub next_leader: Option<String>,
    pub handoff_state: String,
    pub lookahead: Vec<LeaderLookaheadEntry>,
    pub provenance: EventProvenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanksSnapshot {
    pub candidate_banks: Vec<CandidateBankSummary>,
    pub canonical_bank: Option<CandidateBankSummary>,
    pub total_banks_tracked: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentSnapshot {
    pub last_update_parent: Option<UpdateParentSummary>,
    pub total_update_parents: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolSnapshot {
    pub consensus_mode: String,
    pub alpenglow_active: bool,
    pub genesis_slot: Option<u64>,
    pub target_finality_ms: u64,
    pub observed_finality_ms: Option<u64>,
    pub protocol_version: String,
    pub consensus_engine: String,
    pub execution_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalitySnapshot {
    pub mode: String,
    pub finality_latency_ms: Option<u64>,
    pub consensus_finality_ms: Option<u64>,
    pub observation_latency_ms: Option<u64>,
    pub provider_latency_ms: Option<u64>,
    pub chrono_processing_us: u64,
    pub last_finalized_slot: Option<u64>,
    pub cert_type: Option<String>,
    pub stake_percent: Option<f64>,
    pub provenance: EventProvenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitiesSnapshot {
    pub dimensions: HashMap<String, String>,
    pub coverage_score: u32,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSnapshot {
    pub connected: bool,
    pub rpc_endpoint: String,
    pub live_tps: Option<u64>,
    pub active_validators: Option<usize>,
    pub last_update_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySnapshot {
    pub events_received_total: u64,
    pub events_normalized_total: u64,
    pub events_dropped_total: u64,
    pub source_latency_us: u64,
    pub processing_latency_us: u64,
    pub websocket_clients: usize,
}

/// The canonical Chrono Snapshot returned by GET /api/v1/snapshot.
///
/// Contains the complete, bounded consensus state for instantaneous client rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronoSnapshot {
    pub schema_version: u32,
    pub sequence: u64,
    pub snapshot_timestamp_ms: u64,
    pub cluster: String,
    pub source: String,
    pub environment: String, // "live", "local", "fixture"
    pub status: String,      // "LIVE", "STALE", "DISCONNECTED", "DEGRADED"
    pub protocol: ProtocolSnapshot,
    pub slot: SlotSnapshot,
    pub leader: LeaderSnapshot,
    pub banks: BanksSnapshot,
    pub parent: ParentSnapshot,
    pub finality: FinalitySnapshot,
    pub capabilities: CapabilitiesSnapshot,
    pub network: NetworkSnapshot,
    pub telemetry: TelemetrySnapshot,
    pub recent_events: Vec<ChronoServiceEvent>,
}
