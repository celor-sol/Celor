use serde::{Deserialize, Serialize};

/// High-throughput framing protocol between Agave Validator Geyser plugin
/// and Chrono Core over Unix Domain Sockets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronoGeyserRawFrame {
    pub sequence: u64,
    pub observer_validator: String,
    pub observed_at_nanos: u64,
    pub payload: GeyserPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum GeyserPayload {
    /// Slot status update (Processed, Confirmed, Rooted, Dead, CreatedBank)
    SlotStatus {
        slot: u64,
        parent_slot: Option<u64>,
        bank_id: Option<u64>,
        status: String,
    },
    /// Bank-specific lifecycle state change
    BankLifecycle {
        slot: u64,
        bank_id: u64,
        parent_slot: Option<u64>,
        status: String,
        bank_hash: Option<String>,
    },
    /// Alpenglow block footer notification
    BlockFooter {
        slot: u64,
        bank_id: u64,
        bank_hash: String,
        producer_time_nanos: u64,
        user_agent: String,
        final_cert: Option<Vec<u8>>,
        notar_cert: Option<Vec<u8>>,
        skip_cert: Option<Vec<u8>>,
    },
    /// Fast leader handover UpdateParent marker
    UpdateParent {
        slot: u64,
        cleared_bank_id: Option<u64>,
        parent_slot: u64,
        parent_block_id: Option<String>,
        fec_set_index: Option<u32>,
        source: String, // "entry" or "deshred"
    },
    /// Pre-execution deshred transaction
    DeshredTx {
        slot: u64,
        signature: String,
        raw_bytes: Vec<u8>,
        static_accounts: Vec<String>,
        pre_execution_timestamp_nanos: u64,
        fec_set_index: Option<u64>,
    },
    /// Ledger entry notification
    Entry {
        slot: u64,
        bank_id: Option<u64>,
        entry_index: u64,
        tx_count: u64,
        starting_tx_index: u64,
    },
    /// Block metadata notification
    BlockMeta {
        slot: u64,
        blockhash: String,
        parent_slot: u64,
        parent_blockhash: String,
        executed_tx_count: u64,
    },
    /// Ingestor heartbeat for liveness verification
    Heartbeat {
        timestamp_nanos: u64,
        active_slots: u64,
    },
}
