use crate::graph::{BankGraph, BankKey};
use chrono_core::types::{BankId, ProviderId, Slot};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Correlated replacement event recording the succession from an abandoned candidate bank
/// to a new candidate or canonical bank following an UpdateParent instruction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankReplacementRecord {
    pub slot: Slot,
    pub provider_id: ProviderId,
    pub cleared_bank_id: BankId,
    pub cleared_bank_key: Option<BankKey>,
    pub replacement_bank_id: Option<BankId>,
    pub replacement_bank_key: Option<BankKey>,
    pub cleared_at_nanos: u64,
    pub replaced_at_nanos: Option<u64>,
}

/// Replacement Bank Engine tracking bank succession and correlated replacements.
///
/// In Alpenglow / Fast Leader Handover:
/// When a leader switches parent via UpdateParent, the in-flight candidate bank (`cleared_bank_id`)
/// is invalidated and abandoned. A replacement bank is created to continue building on the new parent.
/// Chrono preserves the full lineage:
/// `old bank -> cleared -> replacement bank -> canonical`
/// WITHOUT deleting historical state or confusing provider-local bank IDs.
#[derive(Debug, Default, Clone)]
pub struct ReplacementEngine {
    /// Track pending cleared banks awaiting replacement: (provider, slot) -> cleared bank record
    pending_replacements: HashMap<(ProviderId, Slot), BankReplacementRecord>,
    /// Completed replacement history
    history: Vec<BankReplacementRecord>,
}

impl ReplacementEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records that an UpdateParent event cleared a specific bank.
    pub fn record_cleared_bank(
        &mut self,
        graph: &mut BankGraph,
        provider_id: &ProviderId,
        slot: Slot,
        cleared_bank_id: BankId,
        cleared_at_nanos: u64,
    ) -> BankReplacementRecord {
        // Find existing bank node in graph matching cleared_bank_id
        let cleared_key = graph
            .get_banks_for_slot(slot)
            .into_iter()
            .find(|b| {
                b.identity.provider_id == *provider_id
                    && b.identity.bank_id == Some(cleared_bank_id.clone())
            })
            .map(|b| b.key.clone());

        if let Some(ref k) = cleared_key {
            let reason = format!(
                "Cleared by UpdateParent on slot {} (bank_id: {})",
                slot, cleared_bank_id
            );
            graph.mark_abandoned(k, reason);
        }

        let record = BankReplacementRecord {
            slot,
            provider_id: provider_id.clone(),
            cleared_bank_id,
            cleared_bank_key: cleared_key,
            replacement_bank_id: None,
            replacement_bank_key: None,
            cleared_at_nanos,
            replaced_at_nanos: None,
        };

        self.pending_replacements
            .insert((provider_id.clone(), slot), record.clone());
        record
    }

    /// Correlates a newly observed bank as the replacement bank for a previously cleared bank.
    pub fn correlate_replacement(
        &mut self,
        provider_id: &ProviderId,
        slot: Slot,
        new_bank_id: BankId,
        new_bank_key: BankKey,
        observed_at_nanos: u64,
    ) -> Option<BankReplacementRecord> {
        if let Some(mut record) = self.pending_replacements.remove(&(provider_id.clone(), slot)) {
            // Ensure the newly created bank is different from the cleared bank
            if record.cleared_bank_id != new_bank_id {
                record.replacement_bank_id = Some(new_bank_id);
                record.replacement_bank_key = Some(new_bank_key);
                record.replaced_at_nanos = Some(observed_at_nanos);
                self.history.push(record.clone());
                return Some(record);
            } else {
                // Same bank id; put back into pending
                self.pending_replacements
                    .insert((provider_id.clone(), slot), record);
            }
        }
        None
    }

    /// Returns all completed bank replacement records.
    pub fn completed_replacements(&self) -> &[BankReplacementRecord] {
        &self.history
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono_core::identity::BankIdentity;
    use chrono_core::types::{BankState, Blockhash, ParentReference};
    use crate::graph::BankNode;

    #[test]
    fn test_cleared_bank_correlated_with_replacement_without_deleting_history() {
        let mut graph = BankGraph::new();
        let mut engine = ReplacementEngine::new();
        let provider = ProviderId::new("testnet-validator");
        let slot = Slot(200);

        // 1. Bank A created
        let ident_a = BankIdentity::new(
            provider.clone(),
            Some(BankId(14)),
            slot,
            Some(Blockhash::new("BlockHashA")),
        );
        let node_a = BankNode::new(
            ident_a,
            Some(ParentReference::KnownBlockhash(Blockhash::new("Parent1"))),
            1000,
        );
        let key_a = graph.insert_bank(node_a);

        // 2. UpdateParent clears Bank A (bank_id: 14)
        let cleared_record = engine.record_cleared_bank(
            &mut graph,
            &provider,
            slot,
            BankId(14),
            2000,
        );
        assert_eq!(cleared_record.cleared_bank_key, Some(key_a.clone()));

        // Invariant: Bank A is ABANDONED, not deleted!
        let node_a_check = graph.get_bank(&key_a).expect("Bank A must still exist in graph");
        assert_eq!(node_a_check.state, BankState::Abandoned);

        // 3. Bank B created as replacement bank (bank_id: 15)
        let ident_b = BankIdentity::new(
            provider.clone(),
            Some(BankId(15)),
            slot,
            Some(Blockhash::new("BlockHashB")),
        );
        let node_b = BankNode::new(
            ident_b,
            Some(ParentReference::KnownBlockhash(Blockhash::new("NewParent2"))),
            2100,
        );
        let key_b = graph.insert_bank(node_b);

        // 4. Correlate replacement
        let replacement_record = engine
            .correlate_replacement(&provider, slot, BankId(15), key_b.clone(), 2100)
            .expect("Should correlate replacement bank");

        assert_eq!(replacement_record.cleared_bank_id, BankId(14));
        assert_eq!(replacement_record.replacement_bank_id, Some(BankId(15)));
        assert_eq!(replacement_record.replacement_bank_key, Some(key_b.clone()));

        // 5. Bank B becomes canonical
        graph.mark_canonical(&key_b, 2500);

        let node_b_check = graph.get_bank(&key_b).expect("Bank B must exist");
        assert_eq!(node_b_check.state, BankState::Canonical);

        // Ensure both banks are in the graph for slot 200
        let slot_banks = graph.get_banks_for_slot(slot);
        assert_eq!(slot_banks.len(), 2);
    }
}
