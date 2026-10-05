use chrono_bank::{
    BankGraph, BankNode, CanonicalEvidence, CanonicalResolution, CanonicalResolver,
    FinalityEngine, ParentEngine,
};
use chrono_core::identity::BankIdentity;
use chrono_core::types::{BankId, BankState, Blockhash, ParentReference, ProviderId, Slot};

#[test]
fn test_multiple_banks_per_slot_distinct_nodes() {
    let mut graph = BankGraph::new();
    let provider = ProviderId::new("testnet-validator");
    let slot = Slot(1000);

    // Two candidate banks in the same slot (SIMD-0326 multi-bank)
    let node_a = BankNode::new(
        BankIdentity::new(
            provider.clone(),
            Some(BankId(1)),
            slot,
            Some(Blockhash::new("CandidateHashA")),
        ),
        Some(ParentReference::SlotOnly(Slot(999))),
        1000,
    );
    let key_a = graph.insert_bank(node_a);

    let node_b = BankNode::new(
        BankIdentity::new(
            provider.clone(),
            Some(BankId(2)),
            slot,
            Some(Blockhash::new("CandidateHashB")),
        ),
        Some(ParentReference::SlotOnly(Slot(999))),
        1005,
    );
    let key_b = graph.insert_bank(node_b);

    assert_ne!(key_a, key_b);
    let banks = graph.get_banks_for_slot(slot);
    assert_eq!(banks.len(), 2);
}

#[test]
fn test_same_bank_id_across_providers_never_confused() {
    let mut graph = BankGraph::new();
    let p1 = ProviderId::new("provider-helius");
    let p2 = ProviderId::new("provider-triton");
    let slot = Slot(2000);

    // Both happen to have local bank_id = 42
    let node_1 = BankNode::new(
        BankIdentity::new(
            p1.clone(),
            Some(BankId(42)),
            slot,
            Some(Blockhash::new("BlockhashFromHelius")),
        ),
        None,
        2000,
    );
    let key_1 = graph.insert_bank(node_1);

    let node_2 = BankNode::new(
        BankIdentity::new(
            p2.clone(),
            Some(BankId(42)),
            slot,
            Some(Blockhash::new("BlockhashFromTriton")),
        ),
        None,
        2005,
    );
    let key_2 = graph.insert_bank(node_2);

    assert_ne!(key_1, key_2);
    assert_eq!(graph.total_banks(), 2);

    let retrieved_1 = graph.get_bank(&key_1).unwrap();
    let retrieved_2 = graph.get_bank(&key_2).unwrap();
    assert_eq!(retrieved_1.identity.provider_id, p1);
    assert_eq!(retrieved_2.identity.provider_id, p2);
}

#[test]
fn test_parent_engine_detects_changes_and_missing_parents() {
    let mut graph = BankGraph::new();
    let parent_engine = ParentEngine::new();
    let provider = ProviderId::new("testnet-rpc");
    let slot = Slot(3000);

    // Bank inserted initially with Unknown parent (missing parent)
    let ident = BankIdentity::new(
        provider,
        Some(BankId(1)),
        slot,
        Some(Blockhash::new("ChildHash")),
    );
    let node = BankNode::new(ident, None, 3000);
    let key = graph.insert_bank(node);

    assert!(parent_engine.resolve_parent_blockhash(&graph, &key).is_none());

    // Parent arrives later (out-of-order parent arrival)
    let parent_hash = Blockhash::new("ParentHashAlpha");
    let notification = parent_engine
        .update_parent(
            &mut graph,
            &key,
            ParentReference::KnownBlockhash(parent_hash.clone()),
        )
        .expect("Parent update should emit notification");

    assert_eq!(notification.previous_parent, None);
    assert_eq!(
        notification.new_parent,
        ParentReference::KnownBlockhash(parent_hash.clone())
    );
    assert_eq!(
        parent_engine.resolve_parent_blockhash(&graph, &key),
        Some(parent_hash)
    );
}

#[test]
fn test_out_of_order_lifecycle_convergence() {
    let mut graph = BankGraph::new();
    let resolver = CanonicalResolver::new();
    let mut finality_engine = FinalityEngine::new();
    let provider = ProviderId::new("testnet-rpc");
    let slot = Slot(4000);
    let hash = Blockhash::new("LifecycleHash");

    // 1. Bank arrives (Observed)
    let ident = BankIdentity::new(provider, Some(BankId(1)), slot, Some(hash.clone()));
    let node = BankNode::new(ident, None, 1_000_000);
    let key = graph.insert_bank(node);
    assert_eq!(graph.get_bank(&key).unwrap().state, BankState::Observed);

    // 2. Canonical confirmation arrives later
    let ev = CanonicalEvidence::ConfirmedBlockhash(hash.clone());
    let resolution = resolver.resolve_slot(&mut graph, slot, Some(&ev), 1_100_000);
    assert_eq!(resolution, CanonicalResolution::Resolved(key.clone()));
    assert_eq!(graph.get_bank(&key).unwrap().state, BankState::Canonical);
    assert_eq!(graph.get_bank(&key).unwrap().canonical_latency_nanos(), Some(100_000));

    // 3. Finality root arrives
    let record = finality_engine
        .record_finality(&mut graph, &key, 1_250_000)
        .expect("record finality");
    assert_eq!(graph.get_bank(&key).unwrap().state, BankState::Finalized);
    assert_eq!(record.finality_latency_us, 250); // 250,000ns = 250us
    assert_eq!(record.canonical_latency_us, Some(100)); // 100,000ns = 100us
}

#[test]
fn test_multi_provider_different_bank_id_same_blockhash_correlates() {
    let mut graph = BankGraph::new();
    let p1 = ProviderId::new("provider-helius");
    let p2 = ProviderId::new("provider-triton");
    let slot = Slot(5000);
    let common_hash = Blockhash::new("CanonicalCommonHash5000");

    // Provider A observed local bank_id = 7
    let node_1 = BankNode::new(
        BankIdentity::new(
            p1.clone(),
            Some(BankId(7)),
            slot,
            Some(common_hash.clone()),
        ),
        None,
        5000,
    );
    let key_1 = graph.insert_bank(node_1);

    // Provider B observed local bank_id = 99
    let node_2 = BankNode::new(
        BankIdentity::new(
            p2.clone(),
            Some(BankId(99)),
            slot,
            Some(common_hash.clone()),
        ),
        None,
        5005,
    );
    let key_2 = graph.insert_bank(node_2);

    // Both exist in graph with provider separation
    assert_ne!(key_1, key_2);
    assert_eq!(graph.total_banks(), 2);

    // Cross-provider correlation via blockhash succeeds
    let found = graph.find_by_blockhash(&common_hash);
    assert_eq!(found.len(), 2);
    assert!(found.iter().any(|b| b.key == key_1));
    assert!(found.iter().any(|b| b.key == key_2));
}
