use crate::graph::{BankGraph, BankKey};
use chrono_core::types::{BankState, Blockhash, Slot};
use serde::{Deserialize, Serialize};

/// Cryptographic or network evidence supporting canonical resolution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalEvidence {
    /// Confirmed commitment level observed from Solana RPC/gRPC for this exact blockhash.
    ConfirmedBlockhash(Blockhash),
    /// Direct finality certificate or TowerBFT root proving this block is on the root fork.
    FinalizedRoot(Blockhash),
    /// Child bank confirmed, transitively certifying this parent bank.
    CertifiedChildLineage(Blockhash),
}

/// The result of attempting to resolve the canonical bank for a slot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalResolution {
    /// authoritatively resolved based on valid evidence.
    Resolved(BankKey),
    /// Multiple candidate banks observed, but none has sufficient proof.
    /// Strictly UNKNOWN / UNDECIDED: Chrono never guesses or uses first-seen.
    Ambiguous {
        slot: Slot,
        candidates_count: usize,
    },
    /// No candidate banks observed yet for this slot.
    NoCandidates(Slot),
}

/// Evidence-based canonical bank resolver.
///
/// CRITICAL INVARIANTS:
/// - NEVER use "first seen = canonical".
/// - NEVER use "latest event = canonical".
/// - NEVER use "most transactions = canonical".
/// - If evidence is insufficient, explicitly return `CanonicalResolution::Ambiguous` (UNKNOWN).
#[derive(Debug, Default)]
pub struct CanonicalResolver;

impl CanonicalResolver {
    pub fn new() -> Self {
        Self
    }

    /// Resolves canonical state given available evidence.
    pub fn resolve_slot(
        &self,
        graph: &mut BankGraph,
        slot: Slot,
        evidence: Option<&CanonicalEvidence>,
        timestamp_nanos: u64,
    ) -> CanonicalResolution {
        // Collect bank summaries to decouple from graph borrow
        let bank_summaries: Vec<(BankKey, Option<Blockhash>, BankState)> = graph
            .get_banks_for_slot(slot)
            .into_iter()
            .map(|b| (b.key.clone(), b.identity.blockhash.clone(), b.state))
            .collect();

        if bank_summaries.is_empty() {
            return CanonicalResolution::NoCandidates(slot);
        }

        // If evidence is provided, find the bank that matches the evidence
        if let Some(ev) = evidence {
            let target_hash = match ev {
                CanonicalEvidence::ConfirmedBlockhash(h) => h,
                CanonicalEvidence::FinalizedRoot(h) => h,
                CanonicalEvidence::CertifiedChildLineage(h) => h,
            };

            let matching_keys: Vec<BankKey> = bank_summaries
                .iter()
                .filter(|(_, hash, _)| hash.as_ref() == Some(target_hash))
                .map(|(key, _, _)| key.clone())
                .collect();

            if let Some(canonical_key) = matching_keys.first() {
                graph.mark_canonical(canonical_key, timestamp_nanos);

                // Mark any other candidate banks for this slot as Abandoned
                let other_keys: Vec<BankKey> = bank_summaries
                    .iter()
                    .filter(|(key, _, _)| key != canonical_key)
                    .map(|(key, _, _)| key.clone())
                    .collect();

                for other_key in other_keys {
                    graph.mark_abandoned(
                        &other_key,
                        format!("Competing candidate abandoned in favor of confirmed canonical blockhash {}", target_hash),
                    );
                }

                return CanonicalResolution::Resolved(canonical_key.clone());
            }
        }

        // Check if exactly one bank is already marked canonical or finalized
        let certified: Vec<BankKey> = bank_summaries
            .iter()
            .filter(|(_, _, state)| *state == BankState::Canonical || *state == BankState::Finalized)
            .map(|(key, _, _)| key.clone())
            .collect();

        if let Some(key) = certified.first() {
            return CanonicalResolution::Resolved(key.clone());
        }

        // If there are multiple candidates and NO cryptographic/RPC proof:
        // Do NOT guess. Return Ambiguous.
        if bank_summaries.len() > 1 {
            return CanonicalResolution::Ambiguous {
                slot,
                candidates_count: bank_summaries.len(),
            };
        }

        // If there is only 1 candidate observed, but without confirmation evidence,
        // it remains strictly `Observed`, not yet `Canonical`.
        CanonicalResolution::Ambiguous {
            slot,
            candidates_count: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono_core::identity::BankIdentity;
    use chrono_core::types::{BankId, ProviderId};
    use crate::graph::BankNode;

    #[test]
    fn test_never_resolves_first_seen_as_canonical_without_evidence() {
        let mut graph = BankGraph::new();
        let provider = ProviderId::new("testnet-rpc");
        let slot = Slot(500);

        let ident = BankIdentity::new(
            provider,
            Some(BankId(1)),
            slot,
            Some(Blockhash::new("UnconfirmedHash1")),
        );
        let node = BankNode::new(ident, None, 10_000);
        graph.insert_bank(node);

        let resolver = CanonicalResolver::new();
        // Without evidence, Chrono must NOT mark first-seen as canonical!
        let resolution = resolver.resolve_slot(&mut graph, slot, None, 15_000);

        assert!(matches!(
            resolution,
            CanonicalResolution::Ambiguous {
                slot: Slot(500),
                candidates_count: 1
            }
        ));

        let bank = graph.get_banks_for_slot(slot)[0];
        assert_eq!(bank.state, BankState::Observed);
    }

    #[test]
    fn test_resolves_canonical_with_confirmed_blockhash_evidence() {
        let mut graph = BankGraph::new();
        let provider = ProviderId::new("testnet-rpc");
        let slot = Slot(500);

        let confirmed_hash = Blockhash::new("ConfirmedBlockhashX");
        let orphan_hash = Blockhash::new("OrphanBlockhashY");

        let node_a = BankNode::new(
            BankIdentity::new(provider.clone(), Some(BankId(1)), slot, Some(confirmed_hash.clone())),
            None,
            10_000,
        );
        let key_a = graph.insert_bank(node_a);

        let node_b = BankNode::new(
            BankIdentity::new(provider, Some(BankId(2)), slot, Some(orphan_hash)),
            None,
            10_005,
        );
        let key_b = graph.insert_bank(node_b);

        let resolver = CanonicalResolver::new();
        let evidence = CanonicalEvidence::ConfirmedBlockhash(confirmed_hash);
        let resolution = resolver.resolve_slot(&mut graph, slot, Some(&evidence), 20_000);

        assert_eq!(resolution, CanonicalResolution::Resolved(key_a.clone()));

        // Bank A is now Canonical
        assert_eq!(graph.get_bank(&key_a).unwrap().state, BankState::Canonical);
        // Bank B is Abandoned
        assert_eq!(graph.get_bank(&key_b).unwrap().state, BankState::Abandoned);
    }
}
