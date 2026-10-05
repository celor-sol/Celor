use crate::graph::{BankGraph, BankKey};
use chrono_core::types::{BankId, ParentReference, ProviderId, Slot};
use serde::{Deserialize, Serialize};

/// Result of processing an UpdateParent instruction or event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateParentOutcome {
    /// The bank whose parent was redirected.
    pub target_bank_key: BankKey,
    /// The previous parent reference prior to handover.
    pub old_parent: Option<ParentReference>,
    /// The new authoritative parent reference.
    pub new_parent: ParentReference,
    /// Any candidate banks building on the abandoned parent that were marked abandoned.
    pub abandoned_candidate_keys: Vec<BankKey>,
}

/// Engine executing SIMD-0326 Fast Leader Handover `UpdateParent` state transitions.
#[derive(Debug, Default)]
pub struct UpdateParentEngine;

impl UpdateParentEngine {
    pub fn new() -> Self {
        Self
    }

    /// Applies an UpdateParent instruction to the BankGraph.
    ///
    /// Steps:
    /// 1. Locate the bank for the target slot / provider / bank_id.
    /// 2. If existing child candidate banks were rooted in the previous parent, mark them Abandoned
    ///    with reason "Abandoned by fast leader handover UpdateParent".
    /// 3. Update the parent reference of the target bank.
    /// 4. Return an outcome detailing modified states and abandoned candidates.
    ///
    /// CRITICAL: Old candidate banks are NEVER deleted from the BankGraph.
    pub fn apply_update_parent(
        &self,
        graph: &mut BankGraph,
        provider_id: &ProviderId,
        slot: Slot,
        target_bank_id: Option<BankId>,
        new_parent: ParentReference,
        reason: &str,
    ) -> Option<UpdateParentOutcome> {
        // Find matching bank in the slot
        let target_keys: Vec<BankKey> = graph
            .get_banks_for_slot(slot)
            .into_iter()
            .filter(|b| {
                b.identity.provider_id == *provider_id
                    && (target_bank_id.is_none() || b.identity.bank_id == target_bank_id)
            })
            .map(|b| b.key.clone())
            .collect();

        let target_key = match target_keys.first() {
            Some(k) => k.clone(),
            None => {
                // Target bank not yet registered in graph
                return None;
            }
        };

        let old_parent = graph.get_bank(&target_key)?.parent.clone();

        // Check if other banks in the same slot were speculative candidate forks that are now invalidated
        let mut abandoned_candidates = Vec::new();
        let other_keys: Vec<BankKey> = graph
            .get_banks_for_slot(slot)
            .into_iter()
            .filter(|b| b.key != target_key && b.identity.provider_id == *provider_id)
            .map(|b| b.key.clone())
            .collect();

        for other_key in other_keys {
            let abandon_reason = format!("Abandoned by UpdateParent on slot {}: {}", slot, reason);
            graph.mark_abandoned(&other_key, abandon_reason);
            abandoned_candidates.push(other_key);
        }

        // Update the target bank's parent
        if let Some(target_node) = graph.get_bank_mut(&target_key) {
            target_node.parent = Some(new_parent.clone());
        }

        Some(UpdateParentOutcome {
            target_bank_key: target_key,
            old_parent,
            new_parent,
            abandoned_candidate_keys: abandoned_candidates,
        })
    }

    /// Applies a rich UpdateParent event (from SubscribeUpdateEntryUpdateParent or SubscribeUpdateDeshredUpdateParent).
    ///
    /// Specifically handles `cleared_bank_id` by finding and marking that specific bank abandoned,
    /// and redirecting the remaining or new candidate bank to `parent_block_id` or `parent_slot`.
    #[allow(clippy::too_many_arguments)]
    pub fn apply_rich_update_parent(
        &self,
        graph: &mut BankGraph,
        provider_id: &ProviderId,
        slot: Slot,
        cleared_bank_id: Option<BankId>,
        parent_slot: Slot,
        parent_block_id: Option<String>,
        fec_set_index: Option<u64>,
        source: &str,
    ) -> Option<UpdateParentOutcome> {
        let new_parent = match parent_block_id {
            Some(ref id) => ParentReference::KnownBlockhash(chrono_core::types::Blockhash::new(id)),
            None => ParentReference::SlotOnly(parent_slot),
        };

        // If cleared_bank_id is provided, mark that specific bank as abandoned
        let mut abandoned_candidates = Vec::new();
        if let Some(ref cleared_id) = cleared_bank_id {
            let cleared_keys: Vec<BankKey> = graph
                .get_banks_for_slot(slot)
                .into_iter()
                .filter(|b| {
                    b.identity.provider_id == *provider_id
                        && b.identity.bank_id.as_ref() == Some(cleared_id)
                })
                .map(|b| b.key.clone())
                .collect();

            for k in cleared_keys {
                let reason = format!(
                    "Cleared by UpdateParent ({}) fec_set={:?}: parent redirected to slot {}",
                    source, fec_set_index, parent_slot
                );
                graph.mark_abandoned(&k, reason);
                abandoned_candidates.push(k);
            }
        }

        // Find the active/successor bank for this slot (one that is NOT the cleared bank)
        let active_keys: Vec<BankKey> = graph
            .get_banks_for_slot(slot)
            .into_iter()
            .filter(|b| {
                b.identity.provider_id == *provider_id
                    && (cleared_bank_id.is_none() || b.identity.bank_id != cleared_bank_id)
            })
            .map(|b| b.key.clone())
            .collect();

        if let Some(target_key) = active_keys.first() {
            let old_parent = graph.get_bank(target_key)?.parent.clone();
            if let Some(target_node) = graph.get_bank_mut(target_key) {
                target_node.parent = Some(new_parent.clone());
            }

            Some(UpdateParentOutcome {
                target_bank_key: target_key.clone(),
                old_parent,
                new_parent,
                abandoned_candidate_keys: abandoned_candidates,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono_core::identity::BankIdentity;
    use chrono_core::types::{BankState, Blockhash};
    use crate::graph::BankNode;

    #[test]
    fn test_update_parent_marks_alternative_candidates_abandoned_without_deleting() {
        let mut graph = BankGraph::new();
        let provider = ProviderId::new("testnet-validator-1");
        let slot = Slot(100);

        // Candidate Bank A (optimistic speculative fork)
        let ident_a = BankIdentity::new(
            provider.clone(),
            Some(BankId(1)),
            slot,
            Some(Blockhash::new("HashForkA")),
        );
        let node_a = BankNode::new(
            ident_a,
            Some(ParentReference::KnownBlockhash(Blockhash::new("OldParentHash"))),
            1000,
        );
        let key_a = graph.insert_bank(node_a);

        // Candidate Bank B (the official handover target)
        let ident_b = BankIdentity::new(
            provider.clone(),
            Some(BankId(2)),
            slot,
            Some(Blockhash::new("HashForkB")),
        );
        let node_b = BankNode::new(
            ident_b,
            Some(ParentReference::KnownBlockhash(Blockhash::new("OldParentHash"))),
            1010,
        );
        let key_b = graph.insert_bank(node_b);

        let engine = UpdateParentEngine::new();
        let new_parent = ParentReference::KnownBlockhash(Blockhash::new("AuthoritativeParentHash"));

        let outcome = engine
            .apply_update_parent(
                &mut graph,
                &provider,
                slot,
                Some(BankId(2)),
                new_parent.clone(),
                "Handover to parent B certified",
            )
            .expect("UpdateParent must succeed");

        assert_eq!(outcome.target_bank_key, key_b);
        assert_eq!(outcome.abandoned_candidate_keys, vec![key_a.clone()]);

        // Key invariant: Candidate A is NOT deleted, but marked Abandoned
        let node_a_after = graph.get_bank(&key_a).expect("Bank A must still exist in graph");
        assert_eq!(node_a_after.state, BankState::Abandoned);
        assert!(node_a_after.abandonment_reason.as_ref().unwrap().contains("Abandoned by UpdateParent"));

        // Candidate B has the updated parent
        let node_b_after = graph.get_bank(&key_b).expect("Bank B must exist");
        assert_eq!(node_b_after.parent, Some(new_parent));
    }
}
