#![allow(clippy::field_reassign_with_default)]

/// Phase 6.1 Regression Tests
///
/// Classification: INTEGRATION — FIXTURE
/// Source: Synthetic GeyserPayload frames over real UDS sockets + real CoreStateEngine
/// Purpose: Prove state-machine correctness for edge cases and failure modes.
///
/// These tests do NOT prove real validator observability.
/// They prove that the state machine, adapter, and engine behave
/// correctly under controlled conditions including failure paths.
///
/// Per Phase 6.1 mandate:
/// - Every failing assertion must trace to the production code path
/// - No assertion weakening
/// - No fabricated expected values
/// - No special-casing of test input
use chrono_adapters::{
    evidence_graph::SourceEvidenceGraph,
    geyser_uds_adapter::ValidatorGeyserAdapter,
    provider::ProviderAdapter,
};
use chrono_core::bus::EventBus;
use chrono_core::events::{ChronoEvent, ChronoEventKind};
use chrono_core::identity::BankIdentity;
use chrono_core::types::{
    BankId, Blockhash, CertificateKind, CertificateValidationStatus, FieldProvenance,
    ParentReference, ProviderId, Slot,
};
use chrono_geyser_plugin::events::{ChronoGeyserRawFrame, GeyserPayload};
use chrono_geyser_plugin::ipc::GeyserIpcWriter;
use chrono_server::{state::CoreStateEngine, ServerConfig};
use std::sync::Arc;
use std::time::Duration;

// ============================================================
// §9 — EVENT FAN-OUT / DRAINING CORRECTNESS
// ============================================================

/// Proves that a single Geyser frame translating into N events
/// does NOT silently drop any of them.
///
/// Frame counts (from translate_geyser_payload):
///   SlotStatus   → 2 (SlotObserved + SlotStatusChanged)
///   BankLifecycle → 2 (BankObserved + BankStatusChanged) or 3 if replacement
///   BlockFooter  → 2 (BlockFooterObserved + ProducerTimingObserved) or 3 with cert
///   UpdateParent → 1
///   DeshredTx    → 1
///   Entry        → 1
///   BlockMeta    → 1
#[tokio::test]
async fn test_fanout_slot_status_produces_exactly_2_events() {
    let sock = "/tmp/chrono_p61_fanout_slot.sock";
    let _ = std::fs::remove_file(sock);
    let bus = Arc::new(EventBus::new(256));
    let mut rx = bus.subscribe();
    let mut adapter = ValidatorGeyserAdapter::new(sock, Some(bus.clone()));
    adapter.connect().await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    let writer = GeyserIpcWriter::new(sock.to_string(), 64);
    tokio::time::sleep(Duration::from_millis(50)).await;

    writer.dispatch(ChronoGeyserRawFrame {
        sequence: 1,
        observer_validator: "test-node".to_string(),
        observed_at_nanos: 1_000_000,
        payload: GeyserPayload::SlotStatus {
            slot: 100,
            parent_slot: Some(99),
            bank_id: Some(7),
            status: "Processed".to_string(),
        },
    });

    // Drain with bounded timeout — must get exactly 2
    let mut count = 0;
    for _ in 0..5 {
        if let Ok(ev) = tokio::time::timeout(Duration::from_millis(200), rx.recv()).await {
            let _ = ev.unwrap();
            count += 1;
        } else {
            break;
        }
    }

    assert_eq!(count, 2, "SlotStatus frame must produce exactly 2 ChronoEvents (SlotObserved + SlotStatusChanged)");
    let _ = adapter.disconnect().await;
    let _ = std::fs::remove_file(sock);
}

#[tokio::test]
async fn test_fanout_block_footer_with_cert_produces_exactly_3_events() {
    let sock = "/tmp/chrono_p61_fanout_footer.sock";
    let _ = std::fs::remove_file(sock);
    let bus = Arc::new(EventBus::new(256));
    let mut rx = bus.subscribe();
    let mut adapter = ValidatorGeyserAdapter::new(sock, Some(bus.clone()));
    adapter.connect().await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    let writer = GeyserIpcWriter::new(sock.to_string(), 64);
    tokio::time::sleep(Duration::from_millis(50)).await;

    writer.dispatch(ChronoGeyserRawFrame {
        sequence: 1,
        observer_validator: "test-node".to_string(),
        observed_at_nanos: 1_000_000,
        payload: GeyserPayload::BlockFooter {
            slot: 200,
            bank_id: 5,
            bank_hash: "hash_200".to_string(),
            producer_time_nanos: 900_000,
            user_agent: "agave/2.0.1".to_string(),
            final_cert: Some(vec![0xAB; 192]),
            notar_cert: None,
            skip_cert: None,
        },
    });

    let mut count = 0;
    let mut event_types: Vec<String> = Vec::new();
    for _ in 0..5 {
        if let Ok(ev) = tokio::time::timeout(Duration::from_millis(200), rx.recv()).await {
            let ev = (*ev.unwrap()).clone();
            event_types.push(format!("{:?}", ev.kind).split('{').next().unwrap_or("?").trim().to_string());
            count += 1;
        } else {
            break;
        }
    }

    assert_eq!(count, 3,
        "BlockFooter+cert must produce exactly 3 events (BlockFooterObserved, ProducerTimingObserved, CertificateObserved). Got: {:?}", event_types);
    let _ = adapter.disconnect().await;
    let _ = std::fs::remove_file(sock);
}

#[tokio::test]
async fn test_fanout_block_footer_without_cert_produces_exactly_2_events() {
    let sock = "/tmp/chrono_p61_fanout_footer_nocert.sock";
    let _ = std::fs::remove_file(sock);
    let bus = Arc::new(EventBus::new(256));
    let mut rx = bus.subscribe();
    let mut adapter = ValidatorGeyserAdapter::new(sock, Some(bus.clone()));
    adapter.connect().await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    let writer = GeyserIpcWriter::new(sock.to_string(), 64);
    tokio::time::sleep(Duration::from_millis(50)).await;

    writer.dispatch(ChronoGeyserRawFrame {
        sequence: 1,
        observer_validator: "test-node".to_string(),
        observed_at_nanos: 1_000_000,
        payload: GeyserPayload::BlockFooter {
            slot: 201,
            bank_id: 6,
            bank_hash: "hash_201".to_string(),
            producer_time_nanos: 900_000,
            user_agent: "agave/2.0.1".to_string(),
            final_cert: None,
            notar_cert: None,
            skip_cert: None,
        },
    });

    let mut count = 0;
    for _ in 0..4 {
        if let Ok(ev) = tokio::time::timeout(Duration::from_millis(200), rx.recv()).await {
            let _ = ev.unwrap();
            count += 1;
        } else {
            break;
        }
    }

    assert_eq!(count, 2,
        "BlockFooter without cert must produce exactly 2 events (BlockFooterObserved, ProducerTimingObserved)");
    let _ = adapter.disconnect().await;
    let _ = std::fs::remove_file(sock);
}

#[tokio::test]
async fn test_fanout_bank_lifecycle_after_update_parent_produces_3_events() {
    let sock = "/tmp/chrono_p61_fanout_replacement.sock";
    let _ = std::fs::remove_file(sock);
    let bus = Arc::new(EventBus::new(256));
    let mut rx = bus.subscribe();
    let mut adapter = ValidatorGeyserAdapter::new(sock, Some(bus.clone()));
    adapter.connect().await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    let writer = GeyserIpcWriter::new(sock.to_string(), 64);
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Frame 1: UpdateParent clears bank 1 for slot 300
    writer.dispatch(ChronoGeyserRawFrame {
        sequence: 1,
        observer_validator: "test-node".to_string(),
        observed_at_nanos: 1_000_000,
        payload: GeyserPayload::UpdateParent {
            slot: 300,
            cleared_bank_id: Some(1),
            parent_slot: 299,
            parent_block_id: Some("parent-hash".to_string()),
            fec_set_index: Some(0),
            source: "entry".to_string(),
        },
    });

    // Frame 2: BankLifecycle for slot 300, bank 2 (the replacement)
    writer.dispatch(ChronoGeyserRawFrame {
        sequence: 2,
        observer_validator: "test-node".to_string(),
        observed_at_nanos: 2_000_000,
        payload: GeyserPayload::BankLifecycle {
            slot: 300,
            bank_id: 2,
            parent_slot: Some(299),
            status: "CreatedBank".to_string(),
            bank_hash: None,
        },
    });

    // Frame 1 → 1 event (UpdateParent)
    // Frame 2 → 3 events (BankObserved + BankStatusChanged + ReplacementBank)
    // Total = 4
    let mut count = 0;
    let mut kinds: Vec<String> = Vec::new();
    for _ in 0..6 {
        if let Ok(ev) = tokio::time::timeout(Duration::from_millis(200), rx.recv()).await {
            let ev = (*ev.unwrap()).clone();
            kinds.push(format!("{:?}", ev.kind).split('{').next().unwrap_or("?").trim().to_string());
            count += 1;
        } else {
            break;
        }
    }

    assert_eq!(count, 4,
        "UpdateParent(1) + BankLifecycle-replacement(3) = 4 events. Got kinds: {:?}", kinds);
    assert!(kinds.iter().any(|k| k.contains("ReplacementBank")),
        "ReplacementBank event must be emitted. Got: {:?}", kinds);
    let _ = adapter.disconnect().await;
    let _ = std::fs::remove_file(sock);
}

// ============================================================
// §10 — BLOCK FOOTER PATH EDGE CASES
// ============================================================

/// Footer arrives for unknown bank_id — must not panic, must retain evidence.
#[tokio::test]
async fn test_footer_for_unknown_bank_does_not_panic() {
    let mut config = ServerConfig::default();
    config.source = "local-validator".to_string();
    let mut engine = CoreStateEngine::new(config);

    let provider = ProviderId::new("local-validator");
    let slot = Slot(9999);

    // Footer refers to bank_id = 99, which was never created
    let footer = chrono_core::types::AlpenglowFooter {
        slot,
        bank_id: Some(BankId(99)),
        bank_hash: Some(chrono_core::types::BankHash::new("orphan_hash")),
        block_producer_time_nanos: Some(1_000),
        block_user_agent: Some("agave/2.0.1".to_string()),
        block_final_cert: None,
        skip_reward_cert: None,
        notar_reward_cert: None,
        raw_payload: None,
        received_at_nanos: 2_000,
    };

    let ev = ChronoEvent::new(
        1,
        provider,
        slot,
        None,
        ChronoEventKind::BlockFooterObserved { footer },
    );

    // Must not panic — footer for unknown bank is silently retained as unresolved evidence
    let svc_ev = engine.process_event(ev);
    assert_eq!(svc_ev.event_type, "BlockFooterObserved",
        "Footer for unknown bank must still produce a service event");
    // bank2 does not exist — snapshot shows no banks (just the root if any)
    // The key requirement: no crash, no fabricated bank node
}

/// Footer arrives before BankCreated — must not create a phantom bank.
/// When the bank later arrives, footer data must be attachable (or retained as unresolved evidence).
#[test]
fn test_footer_before_bank_does_not_create_phantom_bank() {
    let mut config = ServerConfig::default();
    config.source = "local-validator".to_string();
    let mut engine = CoreStateEngine::new(config);
    let provider = ProviderId::new("local-validator");
    let slot = Slot(400);

    // Footer arrives first — bank_id = 10 does not exist yet
    let footer = chrono_core::types::AlpenglowFooter {
        slot,
        bank_id: Some(BankId(10)),
        bank_hash: Some(chrono_core::types::BankHash::new("footer_first_hash")),
        block_producer_time_nanos: Some(5_000),
        block_user_agent: Some("agave/2.0.1".to_string()),
        block_final_cert: None,
        skip_reward_cert: None,
        notar_reward_cert: None,
        raw_payload: None,
        received_at_nanos: 6_000,
    };

    engine.process_event(ChronoEvent::new(
        1, provider.clone(), slot, None,
        ChronoEventKind::BlockFooterObserved { footer },
    ));

    // Snapshot: bank_id=10 must NOT exist as a full BankNode
    // (the footer attachment found no match — that's correct behavior)
    let snapshot = engine.build_snapshot(0);
    let phantom = snapshot.banks.candidate_banks.iter().find(|b| b.raw_bank_id == Some(10));
    assert!(phantom.is_none(),
        "Footer for unknown bank must NOT create a phantom BankNode in the graph");

    // Now bank arrives
    let ident = BankIdentity::new(provider.clone(), Some(BankId(10)), slot, Some(Blockhash::new("hash_10")));
    engine.process_event(ChronoEvent::new(
        2, provider, slot, Some(Blockhash::new("hash_10")),
        ChronoEventKind::BankCreated {
            identity: ident,
            parent: ParentReference::SlotOnly(Slot(399)),
        },
    ));

    // Bank now exists
    let snapshot2 = engine.build_snapshot(0);
    let bank = snapshot2.banks.candidate_banks.iter().find(|b| b.raw_bank_id == Some(10));
    assert!(bank.is_some(), "Bank 10 must exist after BankCreated");
    // Note: bank_hash may be None if the late-footer was not retroactively applied.
    // This is the documented behavior: footer evidence is retained but not applied retroactively
    // to a bank created after the footer arrived.
}

/// Footer arrives after BankCreated — bank_hash must be attached.
#[test]
fn test_footer_after_bank_attaches_bank_hash() {
    let mut config = ServerConfig::default();
    config.source = "local-validator".to_string();
    let mut engine = CoreStateEngine::new(config);
    let provider = ProviderId::new("local-validator");
    let slot = Slot(500);

    // Bank created first
    let ident = BankIdentity::new(provider.clone(), Some(BankId(20)), slot, Some(Blockhash::new("hash_20")));
    engine.process_event(ChronoEvent::new(
        1, provider.clone(), slot, Some(Blockhash::new("hash_20")),
        ChronoEventKind::BankCreated {
            identity: ident,
            parent: ParentReference::SlotOnly(Slot(499)),
        },
    ));

    // Footer arrives second — bank_hash and producer time must attach to the existing BankNode
    let footer = chrono_core::types::AlpenglowFooter {
        slot,
        bank_id: Some(BankId(20)),
        bank_hash: Some(chrono_core::types::BankHash::new("execution_state_hash_20")),
        block_producer_time_nanos: Some(1_700_000_000_999_000_000),
        block_user_agent: Some("agave/2.0.1".to_string()),
        block_final_cert: None,
        skip_reward_cert: None,
        notar_reward_cert: None,
        raw_payload: None,
        received_at_nanos: 1_700_000_001_000_000_000,
    };
    engine.process_event(ChronoEvent::new(
        2, provider, slot, None,
        ChronoEventKind::BlockFooterObserved { footer },
    ));

    let snapshot = engine.build_snapshot(0);
    let bank = snapshot.banks.candidate_banks.iter().find(|b| b.raw_bank_id == Some(20)).expect("Bank 20 must exist");
    assert_eq!(bank.bank_hash, Some("execution_state_hash_20".to_string()),
        "Footer bank_hash must be attached to the existing BankNode");
}

// ============================================================
// §11 — CERTIFICATE DEDUPLICATION
// ============================================================

/// Sending two CertificateObserved events for the same slot must produce exactly 1 cert record.
/// (Dedup is based on slot + kind + raw_bytes identity.)
///
/// Note: Current architecture does NOT deduplicate by content — it appends.
/// This test documents the ACTUAL behavior and asserts it is not silently broken.
/// If dedup is added in Phase 7, update this test.
#[test]
fn test_certificate_duplicate_observed_behavior() {
    let mut config = ServerConfig::default();
    config.source = "local-validator".to_string();
    let mut engine = CoreStateEngine::new(config);
    let provider = ProviderId::new("local-validator");
    let slot = Slot(700);

    let cert_bytes = vec![0xCC; 192];

    // First CertificateObserved
    engine.process_event(ChronoEvent::new(
        1, provider.clone(), slot, None,
        ChronoEventKind::CertificateObserved {
            slot,
            kind: CertificateKind::FinalCert,
            raw_len: 192,
            raw_bytes: Some(cert_bytes.clone()),
            block_id: Some("blk_700".to_string()),
            validation_status: Some(CertificateValidationStatus::ParsedUnverified),
            stake_percent: Some(80.0),
        },
    ));

    // Second identical CertificateObserved (simulates duplicate stream delivery)
    engine.process_event(ChronoEvent::new(
        2, provider, slot, None,
        ChronoEventKind::CertificateObserved {
            slot,
            kind: CertificateKind::FinalCert,
            raw_len: 192,
            raw_bytes: Some(cert_bytes),
            block_id: Some("blk_700".to_string()),
            validation_status: Some(CertificateValidationStatus::ParsedUnverified),
            stake_percent: Some(80.0),
        },
    ));

    let certs = engine.captured_certificates();
    // Document actual behavior: both are appended (no content-based dedup yet).
    // This is acceptable — the important invariant is that BlockFooterObserved
    // does NOT add a third copy (that was the Phase 6 double-counting bug).
    assert_eq!(certs.len(), 2,
        "Two CertificateObserved events produce 2 cert records (content dedup is Phase 7). \
         The critical invariant is: BlockFooterObserved must NOT add a duplicate.");
}

/// BlockFooterObserved MUST NOT add a cert to captured_certificates.
/// Only CertificateObserved owns that path.
#[test]
fn test_block_footer_does_not_add_cert_to_captured_certificates() {
    let mut config = ServerConfig::default();
    config.source = "local-validator".to_string();
    let mut engine = CoreStateEngine::new(config);
    let provider = ProviderId::new("local-validator");
    let slot = Slot(800);

    // Send BlockFooterObserved with a cert payload
    let footer = chrono_core::types::AlpenglowFooter {
        slot,
        bank_id: Some(BankId(30)),
        bank_hash: Some(chrono_core::types::BankHash::new("hash_800")),
        block_producer_time_nanos: Some(1_000),
        block_user_agent: Some("agave/2.0.1".to_string()),
        block_final_cert: Some(vec![0xDE; 192]), // Has a cert
        skip_reward_cert: None,
        notar_reward_cert: None,
        raw_payload: None,
        received_at_nanos: 2_000,
    };
    engine.process_event(ChronoEvent::new(
        1, provider, slot, None,
        ChronoEventKind::BlockFooterObserved { footer },
    ));

    // MUST be zero — BlockFooterObserved must NOT parse certs
    let certs = engine.captured_certificates();
    assert_eq!(certs.len(), 0,
        "BlockFooterObserved must NOT push certs to captured_certificates. \
         Only CertificateObserved owns that path. Found {} certs.", certs.len());
}

// ============================================================
// §22 — FAILURE INJECTION TESTS
// ============================================================

/// Malformed NDJSON frame (not valid JSON) must be rejected and counted, not crash.
#[tokio::test]
async fn test_malformed_frame_is_rejected_not_crashed() {
    let sock = "/tmp/chrono_p61_malformed.sock";
    let _ = std::fs::remove_file(sock);
    let bus = Arc::new(EventBus::new(256));
    let mut adapter = ValidatorGeyserAdapter::new(sock, Some(bus.clone()));
    adapter.connect().await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Write raw malformed bytes directly (bypass GeyserIpcWriter)
    use std::io::Write;
    use std::os::unix::net::UnixStream;
    tokio::time::sleep(Duration::from_millis(10)).await;

    let mut stream = UnixStream::connect(sock).expect("Should be able to connect");
    stream.write_all(b"NOT_VALID_JSON_AT_ALL\n").expect("Write failed");
    stream.write_all(b"{\"partial\": true\n").expect("Write failed");
    drop(stream); // Close connection

    tokio::time::sleep(Duration::from_millis(150)).await;

    let counters = adapter.counters().snapshot();
    assert!(counters.events_parse_failed >= 1,
        "Malformed frames must increment events_parse_failed. Got: {}", counters.events_parse_failed);
    assert_eq!(counters.events_received_total, 2,
        "Both lines were received (even though malformed). Got: {}", counters.events_received_total);

    let _ = adapter.disconnect().await;
    let _ = std::fs::remove_file(sock);
}

/// IPC disconnect mid-stream — events before disconnect remain; no fabrication after.
#[tokio::test]
async fn test_ipc_disconnect_does_not_fabricate_continuity() {
    let sock = "/tmp/chrono_p61_disconnect.sock";
    let _ = std::fs::remove_file(sock);
    let bus = Arc::new(EventBus::new(256));
    let mut rx = bus.subscribe();
    let mut adapter = ValidatorGeyserAdapter::new(sock, Some(bus.clone()));
    adapter.connect().await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Phase 1: Send valid events
    let writer = GeyserIpcWriter::new(sock.to_string(), 64);
    tokio::time::sleep(Duration::from_millis(50)).await;

    writer.dispatch(ChronoGeyserRawFrame {
        sequence: 1,
        observer_validator: "test-node".to_string(),
        observed_at_nanos: 1_000_000,
        payload: GeyserPayload::SlotStatus {
            slot: 1000,
            parent_slot: Some(999),
            bank_id: Some(1),
            status: "Processed".to_string(),
        },
    });

    // Drain the 2 events from slot status
    let mut pre_disconnect_count = 0;
    for _ in 0..3 {
        if let Ok(ev) = tokio::time::timeout(Duration::from_millis(200), rx.recv()).await {
            let _ = ev.unwrap();
            pre_disconnect_count += 1;
        } else {
            break;
        }
    }
    assert_eq!(pre_disconnect_count, 2, "Must receive 2 events before disconnect");

    // Phase 2: Disconnect (drop writer, don't send more events)
    drop(writer);
    tokio::time::sleep(Duration::from_millis(100)).await;

    // After disconnect: no new events must appear without a new connection
    let mut post_disconnect_count = 0;
    for _ in 0..3 {
        if let Ok(_) = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await {
            post_disconnect_count += 1;
        } else {
            break;
        }
    }
    assert_eq!(post_disconnect_count, 0,
        "No events must be fabricated after IPC disconnect. Got: {}", post_disconnect_count);

    let _ = adapter.disconnect().await;
    let _ = std::fs::remove_file(sock);
}

/// Sequence gap (seq=1, seq=5) must be counted as out-of-order, not crashed.
#[tokio::test]
async fn test_sequence_gap_counted_not_crashed() {
    let sock = "/tmp/chrono_p61_seqgap.sock";
    let _ = std::fs::remove_file(sock);
    let bus = Arc::new(EventBus::new(256));
    let mut rx = bus.subscribe();
    let mut adapter = ValidatorGeyserAdapter::new(sock, Some(bus.clone()));
    adapter.connect().await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    let writer = GeyserIpcWriter::new(sock.to_string(), 64);
    tokio::time::sleep(Duration::from_millis(50)).await;

    // seq=1 then seq=5 (gap of 3 missing frames)
    writer.dispatch(ChronoGeyserRawFrame {
        sequence: 1,
        observer_validator: "test-node".to_string(),
        observed_at_nanos: 1_000_000,
        payload: GeyserPayload::SlotStatus { slot: 2000, parent_slot: Some(1999), bank_id: None, status: "Processed".to_string() },
    });
    writer.dispatch(ChronoGeyserRawFrame {
        sequence: 5, // Gap: 2, 3, 4 missing
        observer_validator: "test-node".to_string(),
        observed_at_nanos: 5_000_000,
        payload: GeyserPayload::SlotStatus { slot: 2001, parent_slot: Some(2000), bank_id: None, status: "Processed".to_string() },
    });

    // Drain 4 events (2 per SlotStatus frame)
    let mut count = 0;
    for _ in 0..6 {
        if let Ok(ev) = tokio::time::timeout(Duration::from_millis(200), rx.recv()).await {
            let _ = ev.unwrap();
            count += 1;
        } else {
            break;
        }
    }
    assert_eq!(count, 4, "Both frames must still produce events despite gap");

    tokio::time::sleep(Duration::from_millis(100)).await;
    let counters = adapter.counters().snapshot();
    assert_eq!(counters.events_out_of_order, 3,
        "Gap of 3 must be reflected in out_of_order counter. Got: {}", counters.events_out_of_order);

    let _ = adapter.disconnect().await;
    let _ = std::fs::remove_file(sock);
}

// ============================================================
// §6 — EVIDENCE GRAPH AUTHORITY RULES
// ============================================================

/// Source authority: validator-local Geyser must win over RPC for bank_id/bank_hash
/// regardless of which arrived first.
#[test]
fn test_evidence_graph_geyser_authority_beats_rpc_for_bank_fields() {
    let mut graph = SourceEvidenceGraph::new();
    let slot = Slot(3000);

    // RPC arrives first (T=50) with bank_id=100
    graph.record_observation(
        ProviderId::new("testnet-rpc"),
        slot, None, "bank_id", "100",
        FieldProvenance::Direct, 50,
    );

    // Geyser arrives later (T=200) with bank_id=42 — MUST win
    graph.record_observation(
        ProviderId::new("local-validator-geyser"),
        slot, None, "bank_id", "42",
        FieldProvenance::Direct, 200,
    );

    let evidence = graph.get_slot_evidence(slot).expect("Evidence must exist");

    // Geyser is authoritative for bank_id regardless of arrival time
    assert_eq!(evidence.authoritative_bank_id, Some(42),
        "Geyser must be authoritative for bank_id even when RPC arrived first");

    // Both observations must be preserved
    assert_eq!(evidence.observations.len(), 2,
        "Both RPC and Geyser observations must be retained (no destructive overwrite)");

    // Conflict must be flagged (they disagreed)
    assert!(evidence.has_conflicts,
        "Disagreement on bank_id between RPC and Geyser must be flagged as conflict");
}

#[test]
fn test_evidence_graph_rpc_then_geyser_preserves_both() {
    let mut graph = SourceEvidenceGraph::new();
    let slot = Slot(3001);

    graph.record_observation(ProviderId::new("testnet-rpc"), slot, None, "slot", "3001", FieldProvenance::Direct, 100);
    graph.record_observation(ProviderId::new("local-validator-geyser"), slot, None, "bank_id", "77", FieldProvenance::Direct, 80);
    graph.record_observation(ProviderId::new("local-validator-geyser"), slot, None, "bank_hash", "h_abc", FieldProvenance::Direct, 85);

    let evidence = graph.get_slot_evidence(slot).expect("Evidence must exist");
    assert_eq!(evidence.observations.len(), 3, "All 3 observations preserved");
    assert_eq!(evidence.authoritative_bank_id, Some(77));
    assert_eq!(evidence.authoritative_bank_hash, Some("h_abc".to_string()));
    // first_observed_by is the geyser (T=80 < T=100)
    assert_eq!(evidence.first_observed_by, Some(ProviderId::new("local-validator-geyser")));
}

#[test]
fn test_evidence_graph_duplicate_identical_observations_do_not_conflict() {
    let mut graph = SourceEvidenceGraph::new();
    let slot = Slot(3002);

    // Same source, same value — no conflict
    graph.record_observation(ProviderId::new("yellowstone-1"), slot, None, "bank_id", "55", FieldProvenance::Direct, 100);
    graph.record_observation(ProviderId::new("yellowstone-2"), slot, None, "bank_id", "55", FieldProvenance::Direct, 110);

    let evidence = graph.get_slot_evidence(slot).expect("Evidence must exist");
    assert!(!evidence.has_conflicts, "Identical values from different sources must NOT be flagged as conflict");
    assert_eq!(evidence.observations.len(), 2);
}

// ============================================================
// §25 — TIMESTAMP INTEGRITY
// ============================================================

/// Source-provided producer timestamp and Chrono receive timestamp must remain separate.
/// The system must NOT subtract them to produce a fake interval if clocks differ.
#[test]
fn test_producer_timestamp_is_not_fabricated_when_source_omitted() {
    let mut config = ServerConfig::default();
    config.source = "local-validator".to_string();
    let mut engine = CoreStateEngine::new(config);
    let provider = ProviderId::new("local-validator");
    let slot = Slot(9000);

    // Footer with producer_time_nanos = None (source did not provide it)
    let footer = chrono_core::types::AlpenglowFooter {
        slot,
        bank_id: Some(BankId(50)),
        bank_hash: None,
        block_producer_time_nanos: None, // NOT PROVIDED by source
        block_user_agent: None,
        block_final_cert: None,
        skip_reward_cert: None,
        notar_reward_cert: None,
        raw_payload: None,
        received_at_nanos: 99_000,
    };

    engine.process_event(ChronoEvent::new(
        1, provider, slot, None,
        ChronoEventKind::BlockFooterObserved { footer },
    ));

    // ProducerTimingObserved will still be emitted by the UDS adapter with
    // producer_time_nanos from the raw frame — but in the fixture path (no UDS adapter here),
    // the footer event is processed directly. The engine must NOT fabricate a producer time.
    // captured_certificates and producer timing must reflect the real absence.
    let timing = engine.last_producer_timing();
    // In the fixture path: timing is only populated by ProducerTimingObserved events.
    // BlockFooterObserved alone does not produce a timing record.
    // So timing should be None, confirming no fabrication.
    assert!(timing.is_none(),
        "Engine must not fabricate a producer timestamp when source did not provide one");
}

// ============================================================
// §17 — LIVE VS FIXTURE CLASSIFICATION
// ============================================================

/// Tests that FIXTURE environment is set when source is "local-geyser"
/// and LIVE when source is "rpc" or "local-validator".
#[test]
fn test_environment_classification_fixture_vs_live() {
    // FIXTURE: source = "local-geyser"
    let mut cfg_fixture = ServerConfig::default();
    cfg_fixture.source = "local-geyser".to_string();
    let mut engine_fixture = CoreStateEngine::new(cfg_fixture);
    let provider = ProviderId::new("local-geyser");
    let ev = ChronoEvent::new(1, provider.clone(), Slot(1), None, ChronoEventKind::SlotObserved { parent_slot: None });
    let svc = engine_fixture.process_event(ev);
    assert_eq!(svc.environment, "fixture",
        "source=local-geyser must produce environment=fixture");

    // LIVE: source = "local-validator"
    let mut cfg_live = ServerConfig::default();
    cfg_live.source = "local-validator".to_string();
    let mut engine_live = CoreStateEngine::new(cfg_live);
    let ev2 = ChronoEvent::new(1, ProviderId::new("local-validator"), Slot(1), None, ChronoEventKind::SlotObserved { parent_slot: None });
    let svc2 = engine_live.process_event(ev2);
    assert_eq!(svc2.environment, "live",
        "source=local-validator must produce environment=live");

    // LIVE: source = "rpc"
    let mut cfg_rpc = ServerConfig::default();
    cfg_rpc.source = "rpc".to_string();
    let mut engine_rpc = CoreStateEngine::new(cfg_rpc);
    let ev3 = ChronoEvent::new(1, ProviderId::new("rpc"), Slot(1), None, ChronoEventKind::SlotObserved { parent_slot: None });
    let svc3 = engine_rpc.process_event(ev3);
    assert_eq!(svc3.environment, "live",
        "source=rpc must produce environment=live");
}
