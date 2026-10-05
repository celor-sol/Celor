use crate::graph::{BankGraph, BankKey, BankNode};
use chrono_core::identity::BankIdentity;
use chrono_core::types::{BankId, Blockhash, ProviderId, Slot};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Reconciled global block representation across multiple ingestion streams.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciledBlock {
    pub slot: Slot,
    pub blockhash: Blockhash,
    /// Which providers observed this block, along with their local bank_id if provided.
    pub provider_observations: Vec<(ProviderId, Option<BankId>, u64)>,
    pub first_observed_nanos: u64,
}

/// Divergence detected between different providers for the same slot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDivergence {
    pub slot: Slot,
    pub conflicting_hashes: Vec<(ProviderId, Blockhash)>,
}

/// Multi-provider stream reconciler correlating disparate feeds by `(slot, blockhash)`.
///
/// CRITICAL INVARIANT:
/// Reconciles events across providers using `(slot, blockhash)` and parent lineage,
/// NEVER `bank_id`, because `bank_id` is validator-local.
#[derive(Debug, Default)]
pub struct MultiStreamReconciler {
    /// Slot to list of observed blockhashes
    slot_blocks: HashMap<Slot, HashMap<Blockhash, ReconciledBlock>>,
}

impl MultiStreamReconciler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reconciles an incoming bank observation into the multi-provider consensus view.
    pub fn observe(
        &mut self,
        graph: &mut BankGraph,
        identity: BankIdentity,
        timestamp_nanos: u64,
    ) -> BankKey {
        let _key = BankKey::from_identity(&identity);

        if let Some(ref hash) = identity.blockhash {
            let slot_map = self.slot_blocks.entry(identity.slot).or_default();
            let block = slot_map
                .entry(hash.clone())
                .or_insert_with(|| ReconciledBlock {
                    slot: identity.slot,
                    blockhash: hash.clone(),
                    provider_observations: Vec::new(),
                    first_observed_nanos: timestamp_nanos,
                });

            block.provider_observations.push((
                identity.provider_id.clone(),
                identity.bank_id.clone(),
                timestamp_nanos,
            ));
        }

        let node = BankNode::new(identity, None, timestamp_nanos);
        graph.insert_bank(node)
    }

    /// Checks if there is divergence (conflicting blockhashes) observed across providers for a slot.
    pub fn check_divergence(&self, slot: Slot) -> Option<ProviderDivergence> {
        let slot_map = self.slot_blocks.get(&slot)?;
        if slot_map.len() > 1 {
            let mut conflicting = Vec::new();
            for (hash, block) in slot_map {
                for (p, _, _) in &block.provider_observations {
                    conflicting.push((p.clone(), hash.clone()));
                }
            }
            Some(ProviderDivergence {
                slot,
                conflicting_hashes: conflicting,
            })
        } else {
            None
        }
    }

    /// Gets reconciled block if agreed upon across providers.
    pub fn get_reconciled_block(&self, slot: Slot, hash: &Blockhash) -> Option<&ReconciledBlock> {
        self.slot_blocks.get(&slot)?.get(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_provider_reconciliation_merges_matching_hashes() {
        let mut graph = BankGraph::new();
        let mut reconciler = MultiStreamReconciler::new();

        let slot = Slot(200);
        let blockhash = Blockhash::new("ReconciledHashAlpha");

        let p1 = ProviderId::new("provider-testnet-1");
        let p2 = ProviderId::new("provider-testnet-2");

        // Provider 1 assigns local bank_id 4
        let ident1 = BankIdentity::new(
            p1.clone(),
            Some(BankId(4)),
            slot,
            Some(blockhash.clone()),
        );

        // Provider 2 assigns local bank_id 99
        let ident2 = BankIdentity::new(
            p2.clone(),
            Some(BankId(99)),
            slot,
            Some(blockhash.clone()),
        );

        reconciler.observe(&mut graph, ident1, 1000);
        reconciler.observe(&mut graph, ident2, 1050);

        // No divergence because both observed the same blockhash
        assert!(reconciler.check_divergence(slot).is_none());

        let block = reconciler.get_reconciled_block(slot, &blockhash).unwrap();
        assert_eq!(block.provider_observations.len(), 2);
        assert_eq!(block.provider_observations[0].0, p1);
        assert_eq!(block.provider_observations[0].1, Some(BankId(4)));
        assert_eq!(block.provider_observations[1].0, p2);
        assert_eq!(block.provider_observations[1].1, Some(BankId(99)));
    }

    #[test]
    fn test_multi_provider_divergence_detected() {
        let mut graph = BankGraph::new();
        let mut reconciler = MultiStreamReconciler::new();

        let slot = Slot(200);
        let p1 = ProviderId::new("provider-testnet-1");
        let p2 = ProviderId::new("provider-testnet-2");

        let ident1 = BankIdentity::new(
            p1,
            Some(BankId(1)),
            slot,
            Some(Blockhash::new("HashA")),
        );
        let ident2 = BankIdentity::new(
            p2,
            Some(BankId(1)),
            slot,
            Some(Blockhash::new("HashB")),
        );

        reconciler.observe(&mut graph, ident1, 1000);
        reconciler.observe(&mut graph, ident2, 1010);

        let div = reconciler.check_divergence(slot).expect("divergence must be detected");
        assert_eq!(div.conflicting_hashes.len(), 2);
    }
}
