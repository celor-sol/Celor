use crate::identity::BankIdentity;
use crate::types::{
    AlpenglowFooter, BankId, BankState, BankStatus, Blockhash, CertificateKind,
    CertificateValidationStatus, LeaderId, ObserverContext, ParentReference, ProviderId, Slot,
};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Normalized Chrono Internal Event.
///
/// Encapsulates raw stream notifications into vendor-neutral protocol events.
/// Every event carries a monotonic hardware timestamp (`received_instant`)
/// for non-wallclock latency calculations alongside serializable telemetry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronoEvent {
    pub event_id: u64,
    pub provider: ProviderId,
    pub slot: Slot,
    pub blockhash: Option<Blockhash>,
    pub received_time_ms: u64,
    #[serde(skip, default = "Instant::now")]
    pub received_instant: Instant,
    pub kind: ChronoEventKind,
    #[serde(default)]
    pub observer: Option<ObserverContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChronoEventKind {
    /// New slot boundary or progression observed on provider stream.
    SlotObserved {
        parent_slot: Option<Slot>,
    },
    /// Slot status changed (Processed, Confirmed, Rooted, Dead, etc.).
    SlotStatusChanged {
        slot: Slot,
        status: String,
        bank_id: Option<BankId>,
    },
    /// Leader identity observed or scheduled for slot.
    LeaderObserved {
        leader: LeaderId,
    },
    /// Leader schedule batch observed for a range of slots.
    LeaderScheduleObserved {
        starting_slot: Slot,
        leaders: Vec<LeaderId>,
    },
    /// Candidate bank observed (with optional provider-local bank_id).
    BankObserved {
        identity: BankIdentity,
        parent: ParentReference,
        state: BankState,
    },
    /// Bank creation event observed directly (e.g. from Yellowstone / Geyser).
    BankCreated {
        identity: BankIdentity,
        parent: ParentReference,
    },
    /// Bank lifecycle status changed (CreatedBank, Processed, Confirmed, Rooted, Dead).
    BankStatusChanged {
        identity: BankIdentity,
        status: BankStatus,
    },
    /// Transaction observed on stream (pre-execution or confirmed).
    TransactionObserved {
        slot: Slot,
        bank_id: Option<BankId>,
        signature: String,
    },
    /// Transaction executed with execution metadata.
    TransactionExecuted {
        slot: Slot,
        bank_id: Option<BankId>,
        signature: String,
        error: Option<String>,
    },
    /// Entry observed (entry stream).
    EntryObserved {
        slot: Slot,
        bank_id: Option<BankId>,
        entry_index: u64,
        tx_count: u64,
    },
    /// Fast leader handover ParentChanged / UpdateParent marker received.
    ParentChanged {
        abandoned_bank: BankIdentity,
        new_parent: ParentReference,
        reason: String,
    },
    /// UpdateParent notification from entry stream or deshred stream.
    UpdateParent {
        slot: Slot,
        cleared_bank_id: Option<BankId>,
        parent_slot: Slot,
        parent_block_id: Option<String>,
        fec_set_index: Option<u64>,
        source: String,
    },
    /// Bank abandoned / orphaned without deletion.
    BankAbandoned {
        identity: BankIdentity,
        reason: String,
    },
    /// Correlated replacement bank created after UpdateParent.
    ReplacementBank {
        old_identity: BankIdentity,
        replacement_identity: BankIdentity,
        slot: Slot,
    },
    /// Block completed and sealed with deterministic blockhash.
    BlockObserved {
        blockhash: Blockhash,
        parent_blockhash: Option<Blockhash>,
        tx_count: u64,
    },
    /// Block metadata observed from SubscribeUpdateBlockMeta.
    BlockMetaObserved {
        slot: Slot,
        blockhash: Blockhash,
        parent_slot: Slot,
        parent_blockhash: Blockhash,
        bank_id: Option<BankId>,
    },
    /// Alpenglow block footer observed with timing and certificates.
    BlockFooterObserved {
        footer: AlpenglowFooter,
    },
    /// Consensus certificate observed.
    CertificateObserved {
        slot: Slot,
        kind: CertificateKind,
        raw_len: usize,
        #[serde(default)]
        raw_bytes: Option<Vec<u8>>,
        #[serde(default)]
        block_id: Option<String>,
        #[serde(default)]
        validation_status: Option<CertificateValidationStatus>,
        #[serde(default)]
        stake_percent: Option<f64>,
    },
    /// Canonical branch resolution confirmed for slot.
    CanonicalObserved {
        canonical_identity: BankIdentity,
    },
    /// Finality achieved (Fast Path BLS certificate or TowerBFT root).
    FinalizedObserved {
        identity: BankIdentity,
        cert_type: String,
        latency_ms: Option<u64>,
    },
    /// Pre-execution deshred transaction observed.
    DeshredObserved {
        slot: Slot,
        signature: String,
        #[serde(default)]
        raw_tx_bytes: Option<Vec<u8>>,
        #[serde(default)]
        pre_execution_timestamp_nanos: Option<u64>,
        #[serde(default)]
        static_accounts: Vec<String>,
        #[serde(default)]
        fec_set_index: Option<u64>,
    },
    /// High-precision producer nanosecond timing observation.
    ProducerTimingObserved {
        slot: Slot,
        producer_time_nanos: u64,
        chrono_received_at_nanos: u64,
        interval_nanos: i64,
    },
    /// Provider connection status changes.
    ProviderConnected {
        provider: ProviderId,
    },
    ProviderDisconnected {
        provider: ProviderId,
        reason: String,
    },
}

impl ChronoEvent {
    pub fn new(
        event_id: u64,
        provider: ProviderId,
        slot: Slot,
        blockhash: Option<Blockhash>,
        kind: ChronoEventKind,
    ) -> Self {
        Self {
            event_id,
            provider,
            slot,
            blockhash,
            received_time_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            received_instant: Instant::now(),
            kind,
            observer: None,
        }
    }

    pub fn with_observer(mut self, observer: ObserverContext) -> Self {
        self.observer = Some(observer);
        self
    }
}
