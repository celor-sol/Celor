use crate::types::{
    HandoffState, LeaderId, ProtocolProfile, ProviderId, Slot, SlotDuration,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

/// Cluster connection and identity state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterState {
    pub name: String,
    pub rpc_url: String,
    pub ws_url: String,
    pub genesis_hash: Option<String>,
    pub solana_core_version: Option<String>,
    pub feature_set: Option<u64>,
}

/// Provider connection health state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderConnectionState {
    Connected,
    Degraded,
    Reconnecting,
    Disconnected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderState {
    pub id: ProviderId,
    pub state: ProviderConnectionState,
    pub messages_received: u64,
    pub reconnect_count: u64,
    pub last_seen_ms: u64,
}

/// In-memory snapshot of current consensus state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusStateSnapshot {
    pub current_slot: Slot,
    pub target_slot_duration_ms: u64,
    pub current_leader: Option<LeaderId>,
    pub next_leader: Option<LeaderId>,
    pub handoff_state: HandoffState,
    pub protocol: ProtocolProfile,
    pub candidate_banks_count: usize,
    pub canonical_bank_id: Option<String>,
    pub canonical_blockhash: Option<String>,
    pub finality_latency_ms: Option<u64>,
    pub provider_state: ProviderConnectionState,
}

/// Thread-safe in-memory state store for CHRONO Core.
///
/// Contains cluster state, protocol profile, clock state, leader state,
/// provider connection state, and finality state.
pub struct InMemoryStateStore {
    cluster: RwLock<ClusterState>,
    protocol: RwLock<ProtocolProfile>,
    current_slot: RwLock<Slot>,
    slot_duration: RwLock<SlotDuration>,
    current_leader: RwLock<Option<LeaderId>>,
    next_leader: RwLock<Option<LeaderId>>,
    handoff_state: RwLock<HandoffState>,
    leader_schedule: RwLock<HashMap<Slot, LeaderId>>,
    providers: RwLock<HashMap<ProviderId, ProviderState>>,
    canonical_blockhashes: RwLock<HashMap<Slot, String>>,
    finality_latencies: RwLock<HashMap<Slot, u64>>,
}

impl InMemoryStateStore {
    pub fn new(cluster_name: impl Into<String>, rpc_url: impl Into<String>, ws_url: impl Into<String>) -> Self {
        Self {
            cluster: RwLock::new(ClusterState {
                name: cluster_name.into(),
                rpc_url: rpc_url.into(),
                ws_url: ws_url.into(),
                genesis_hash: None,
                solana_core_version: None,
                feature_set: None,
            }),
            protocol: RwLock::new(ProtocolProfile::default()),
            current_slot: RwLock::new(Slot::ZERO),
            slot_duration: RwLock::new(SlotDuration::MS_250),
            current_leader: RwLock::new(None),
            next_leader: RwLock::new(None),
            handoff_state: RwLock::new(HandoffState::Unknown),
            leader_schedule: RwLock::new(HashMap::new()),
            providers: RwLock::new(HashMap::new()),
            canonical_blockhashes: RwLock::new(HashMap::new()),
            finality_latencies: RwLock::new(HashMap::new()),
        }
    }

    pub fn set_cluster_genesis(&self, genesis: String, version: String, feature_set: u64) {
        if let Ok(mut c) = self.cluster.write() {
            c.genesis_hash = Some(genesis);
            c.solana_core_version = Some(version);
            c.feature_set = Some(feature_set);
        }
    }

    pub fn set_protocol(&self, profile: ProtocolProfile) {
        if let Ok(mut p) = self.protocol.write() {
            *p = profile;
        }
    }

    pub fn set_current_slot(&self, slot: Slot) {
        if let Ok(mut s) = self.current_slot.write() {
            if slot > *s {
                *s = slot;
            }
        }
    }

    pub fn get_current_slot(&self) -> Slot {
        self.current_slot.read().map(|s| *s).unwrap_or(Slot::ZERO)
    }

    pub fn set_slot_duration(&self, duration: SlotDuration) {
        if let Ok(mut d) = self.slot_duration.write() {
            *d = duration;
        }
    }

    pub fn get_slot_duration(&self) -> SlotDuration {
        self.slot_duration.read().map(|d| *d).unwrap_or(SlotDuration::MS_250)
    }

    pub fn set_leaders(&self, current: Option<LeaderId>, next: Option<LeaderId>, handoff: HandoffState) {
        if let Ok(mut l) = self.current_leader.write() {
            *l = current;
        }
        if let Ok(mut n) = self.next_leader.write() {
            *n = next;
        }
        if let Ok(mut h) = self.handoff_state.write() {
            *h = handoff;
        }
    }

    pub fn cache_leader_schedule(&self, schedule: HashMap<Slot, LeaderId>) {
        if let Ok(mut map) = self.leader_schedule.write() {
            map.extend(schedule);
        }
    }

    pub fn get_leader_for_slot(&self, slot: Slot) -> Option<LeaderId> {
        self.leader_schedule.read().ok()?.get(&slot).cloned()
    }

    pub fn update_provider_state(&self, id: ProviderId, state: ProviderConnectionState) {
        if let Ok(mut map) = self.providers.write() {
            let entry = map.entry(id.clone()).or_insert_with(|| ProviderState {
                id,
                state,
                messages_received: 0,
                reconnect_count: 0,
                last_seen_ms: 0,
            });
            entry.state = state;
            entry.last_seen_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
        }
    }

    pub fn record_canonical_blockhash(&self, slot: Slot, hash: String) {
        if let Ok(mut map) = self.canonical_blockhashes.write() {
            map.insert(slot, hash);
        }
    }

    pub fn record_finality_latency(&self, slot: Slot, latency_ms: u64) {
        if let Ok(mut map) = self.finality_latencies.write() {
            map.insert(slot, latency_ms);
        }
    }

    pub fn snapshot(&self) -> ConsensusStateSnapshot {
        let slot = self.get_current_slot();
        let target_duration = self.get_slot_duration();
        let current_l = self.current_leader.read().ok().and_then(|l| l.clone());
        let next_l = self.next_leader.read().ok().and_then(|l| l.clone());
        let handoff = self.handoff_state.read().map(|h| *h).unwrap_or(HandoffState::Unknown);
        let proto = self.protocol.read().map(|p| p.clone()).unwrap_or_default();
        let canonical_hash = self.canonical_blockhashes.read().ok().and_then(|map| map.get(&slot).cloned());
        let latency = self.finality_latencies.read().ok().and_then(|map| map.get(&slot).copied());

        let prov_state = self.providers.read().ok()
            .and_then(|map| map.values().next().map(|p| p.state))
            .unwrap_or(ProviderConnectionState::Disconnected);

        ConsensusStateSnapshot {
            current_slot: slot,
            target_slot_duration_ms: target_duration.as_millis(),
            current_leader: current_l,
            next_leader: next_l,
            handoff_state: handoff,
            protocol: proto,
            candidate_banks_count: 1,
            canonical_bank_id: None,
            canonical_blockhash: canonical_hash,
            finality_latency_ms: latency,
            provider_state: prov_state,
        }
    }
}
