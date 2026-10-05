use chrono_core::identity::BankIdentity;
use chrono_core::types::{
    AlpenglowFooter, BankHash, BankId, BankState, BankStatus, Blockhash, ParentReference,
    ProviderId, Slot,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

/// Unique identifier for a bank node within the in-memory BankGraph.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BankKey {
    pub provider_id: ProviderId,
    pub slot: Slot,
    pub bank_id: Option<BankId>,
    pub blockhash: Option<Blockhash>,
}

impl BankKey {
    pub fn from_identity(identity: &BankIdentity) -> Self {
        Self {
            provider_id: identity.provider_id.clone(),
            slot: identity.slot,
            bank_id: identity.bank_id.clone(),
            blockhash: identity.blockhash.clone(),
        }
    }
}

/// A candidate or confirmed bank in the Solana cluster.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankNode {
    pub key: BankKey,
    pub identity: BankIdentity,
    pub parent: Option<ParentReference>,
    /// Optional validator node identity producing or reporting this bank.
    pub node_identity: Option<String>,
    /// Cryptographic state accumulator of the bank (distinct from Blockhash).
    pub bank_hash: Option<BankHash>,
    /// Timestamp reported by the block producer in nanoseconds.
    pub producer_time_nanos: Option<u64>,
    /// Software version or client user-agent of the block producer.
    pub producer_user_agent: Option<String>,
    /// Raw bank lifecycle status from provider (e.g. CreatedBank, Processed, Rooted).
    pub bank_status: BankStatus,
    /// Attached Alpenglow block footer containing certificates and execution telemetry.
    pub footer: Option<AlpenglowFooter>,
    /// High-resolution monotonic timestamp (nanoseconds) when this bank was first observed.
    pub first_observed_nanos: u64,
    /// Current consensus state of this bank.
    pub state: BankState,
    /// Reason if abandoned (e.g. "UpdateParent handover", "fork pruned").
    pub abandonment_reason: Option<String>,
    /// High-resolution monotonic timestamp (nanoseconds) when marked canonical.
    pub canonical_at_nanos: Option<u64>,
    /// High-resolution monotonic timestamp (nanoseconds) when certified finalized.
    pub finalized_at_nanos: Option<u64>,
}

impl BankNode {
    pub fn new(
        identity: BankIdentity,
        parent: Option<ParentReference>,
        observed_nanos: u64,
    ) -> Self {
        let key = BankKey::from_identity(&identity);
        Self {
            key,
            identity,
            parent,
            node_identity: None,
            bank_hash: None,
            producer_time_nanos: None,
            producer_user_agent: None,
            bank_status: BankStatus::Unknown,
            footer: None,
            first_observed_nanos: observed_nanos,
            state: BankState::Observed,
            abandonment_reason: None,
            canonical_at_nanos: None,
            finalized_at_nanos: None,
        }
    }

    /// Attaches an Alpenglow block footer to this bank node.
    pub fn attach_footer(&mut self, footer: AlpenglowFooter) {
        if footer.bank_hash.is_some() {
            self.bank_hash = footer.bank_hash.clone();
        }
        if footer.block_producer_time_nanos.is_some() {
            self.producer_time_nanos = footer.block_producer_time_nanos;
        }
        if footer.block_user_agent.is_some() {
            self.producer_user_agent = footer.block_user_agent.clone();
        }
        self.footer = Some(footer);
    }

    /// Updates the raw validator bank status.
    pub fn update_bank_status(&mut self, status: BankStatus) {
        self.bank_status = status;
        match status {
            BankStatus::Rooted => {
                self.state = BankState::Finalized;
            }
            BankStatus::Dead => {
                self.state = BankState::Abandoned;
                if self.abandonment_reason.is_none() {
                    self.abandonment_reason = Some("Bank marked Dead by validator".to_string());
                }
            }
            _ => {}
        }
    }

    /// Latency from block producer creation time to observer receipt (nanoseconds).
    pub fn producer_to_observer_latency_nanos(&self) -> Option<u64> {
        self.producer_time_nanos
            .map(|prod| self.first_observed_nanos.saturating_sub(prod))
    }

    /// Latency from initial observation to canonical certification.
    pub fn canonical_latency_nanos(&self) -> Option<u64> {
        self.canonical_at_nanos
            .map(|c| c.saturating_sub(self.first_observed_nanos))
    }

    /// Latency from initial observation to cryptographic finality.
    pub fn finality_latency_nanos(&self) -> Option<u64> {
        self.finalized_at_nanos
            .map(|f| f.saturating_sub(self.first_observed_nanos))
    }
}

/// In-memory directed acyclic graph of candidate and canonical banks.
///
/// CRITICAL INVARIANTS:
/// 1. Never fabricate forks: Only nodes backed by actual provider evidence exist.
/// 2. Respect provider-local bank_id: BankId is scoped to ProviderId.
/// 3. Never delete abandoned banks: Retain state transitions for historical inspection.
#[derive(Debug, Default, Clone)]
pub struct BankGraph {
    /// Slot to list of bank keys for that slot.
    slot_index: BTreeMap<Slot, Vec<BankKey>>,
    /// Bank key to bank node.
    nodes: HashMap<BankKey, BankNode>,
    /// Index from blockhash to bank keys (for cross-provider or parent resolution).
    blockhash_index: HashMap<Blockhash, Vec<BankKey>>,
}

impl BankGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a new bank observation into the graph.
    /// If the exact bank already exists, returns the existing key without duplicating.
    pub fn insert_bank(&mut self, node: BankNode) -> BankKey {
        let key = node.key.clone();

        if let Some(existing) = self.nodes.get_mut(&key) {
            // Update parent or blockhash if it was previously unknown
            if existing.parent.is_none() && node.parent.is_some() {
                existing.parent = node.parent;
            }
            if existing.identity.blockhash.is_none() && node.identity.blockhash.is_some() {
                existing.identity.blockhash = node.identity.blockhash.clone();
                existing.key.blockhash = node.identity.blockhash.clone();
                if let Some(ref hash) = existing.identity.blockhash {
                    self.blockhash_index
                        .entry(hash.clone())
                        .or_default()
                        .push(key.clone());
                }
            }
            return key;
        }

        let slot = node.identity.slot;
        if let Some(ref hash) = node.identity.blockhash {
            self.blockhash_index
                .entry(hash.clone())
                .or_default()
                .push(key.clone());
        }

        self.slot_index.entry(slot).or_default().push(key.clone());
        self.nodes.insert(key.clone(), node);

        key
    }

    /// Retrieves a bank node by key.
    pub fn get_bank(&self, key: &BankKey) -> Option<&BankNode> {
        self.nodes.get(key)
    }

    /// Retrieves a mutable reference to a bank node by key.
    pub fn get_bank_mut(&mut self, key: &BankKey) -> Option<&mut BankNode> {
        self.nodes.get_mut(key)
    }

    /// Retrieves all bank nodes observed for a specific slot.
    pub fn get_banks_for_slot(&self, slot: Slot) -> Vec<&BankNode> {
        self.slot_index
            .get(&slot)
            .map(|keys| keys.iter().filter_map(|k| self.nodes.get(k)).collect())
            .unwrap_or_default()
    }

    /// Returns the total count of bank nodes in the graph.
    pub fn total_banks(&self) -> usize {
        self.nodes.len()
    }

    /// Marks a bank as Canonical (winning candidate for its slot).
    pub fn mark_canonical(&mut self, key: &BankKey, timestamp_nanos: u64) -> bool {
        if let Some(node) = self.nodes.get_mut(key) {
            node.state = BankState::Canonical;
            node.canonical_at_nanos = Some(timestamp_nanos);
            true
        } else {
            false
        }
    }

    /// Marks a candidate bank as Abandoned.
    /// Crucial: The bank is NOT deleted; its state and reason are recorded for forensics.
    pub fn mark_abandoned(&mut self, key: &BankKey, reason: impl Into<String>) -> bool {
        if let Some(node) = self.nodes.get_mut(key) {
            node.state = BankState::Abandoned;
            node.abandonment_reason = Some(reason.into());
            true
        } else {
            false
        }
    }

    /// Marks a candidate bank matching (slot, bank_id) as Abandoned without deleting.
    pub fn mark_abandoned_by_id(&mut self, slot: Slot, bank_id: &BankId, reason: impl Into<String>) -> bool {
        let reason_str = reason.into();
        let mut found = false;
        if let Some(keys) = self.slot_index.get(&slot).cloned() {
            for key in keys {
                if let Some(node) = self.nodes.get_mut(&key) {
                    if node.identity.bank_id.as_ref() == Some(bank_id) {
                        node.state = BankState::Abandoned;
                        node.abandonment_reason = Some(reason_str.clone());
                        found = true;
                    }
                }
            }
        }
        found
    }

    /// Marks a bank as Finalized (cryptographically certified).
    pub fn mark_finalized(&mut self, key: &BankKey, timestamp_nanos: u64) -> bool {
        if let Some(node) = self.nodes.get_mut(key) {
            node.state = BankState::Finalized;
            node.finalized_at_nanos = Some(timestamp_nanos);
            true
        } else {
            false
        }
    }

    /// Finds bank keys matching a given blockhash across any provider.
    pub fn find_by_blockhash(&self, hash: &Blockhash) -> Vec<&BankNode> {
        self.blockhash_index
            .get(hash)
            .map(|keys| keys.iter().filter_map(|k| self.nodes.get(k)).collect())
            .unwrap_or_default()
    }

    /// Prunes slots older than `retention_slots` behind the highest observed slot
    /// to bound in-memory footprint while preserving active consensus history.
    pub fn prune_older_than(&mut self, min_slot: Slot) -> usize {
        let mut pruned = 0;
        let old_slots: Vec<Slot> = self
            .slot_index
            .range(..min_slot)
            .map(|(s, _)| *s)
            .collect();

        for s in old_slots {
            if let Some(keys) = self.slot_index.remove(&s) {
                for k in keys {
                    if let Some(node) = self.nodes.remove(&k) {
                        pruned += 1;
                        if let Some(ref hash) = node.identity.blockhash {
                            if let Some(hashes) = self.blockhash_index.get_mut(hash) {
                                hashes.retain(|existing_k| existing_k != &k);
                            }
                        }
                    }
                }
            }
        }
        pruned
    }
}
