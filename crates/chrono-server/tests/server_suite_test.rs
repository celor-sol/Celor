use chrono_core::events::{ChronoEvent, ChronoEventKind};
use chrono_core::identity::BankIdentity;
use chrono_core::types::{
    BankId, Blockhash, CertificateKind, ParentReference, ProviderId, Slot,
};
use chrono_server::{
    config::ServerConfig,
    envelope::EventProvenance,
    service::ChronoServer,
    state::CoreStateEngine,
};

#[test]
fn test_sequence_allocation_strictly_increasing() {
    let mut config = ServerConfig::default();
    config.source = "local-geyser".to_string();
    let mut engine = CoreStateEngine::new(config);

    let provider = ProviderId::new("test-provider");
    let ev1 = ChronoEvent::new(1, provider.clone(), Slot(100), None, ChronoEventKind::SlotObserved { parent_slot: Some(Slot(99)) });
    let ev2 = ChronoEvent::new(2, provider.clone(), Slot(101), None, ChronoEventKind::SlotObserved { parent_slot: Some(Slot(100)) });
    let ev3 = ChronoEvent::new(3, provider.clone(), Slot(102), None, ChronoEventKind::SlotObserved { parent_slot: Some(Slot(101)) });

    let s1 = engine.process_event(ev1);
    let s2 = engine.process_event(ev2);
    let s3 = engine.process_event(ev3);

    assert_eq!(s1.sequence, 1);
    assert_eq!(s2.sequence, 2);
    assert_eq!(s3.sequence, 3);
    assert!(s3.sequence > s2.sequence);
    assert!(s2.sequence > s1.sequence);
}

#[test]
fn test_event_replay_and_gap_detection() {
    let mut config = ServerConfig::default();
    config.event_buffer_capacity = 4; // Bounded ring buffer of 4
    let mut engine = CoreStateEngine::new(config);

    let provider = ProviderId::new("test-provider");

    // Push 5 events (events 1, 2, 3, 4, 5)
    for i in 1..=5 {
        let ev = ChronoEvent::new(
            i,
            provider.clone(),
            Slot(100 + i),
            None,
            ChronoEventKind::SlotObserved { parent_slot: None },
        );
        engine.process_event(ev);
    }

    // Sequence numbers allocated: 1, 2, 3, 4, 5.
    // Buffer capacity is 4, so oldest retained is 2 (event 1 was evicted).
    // Test 1: Replay from sequence 3 -> should return events 4 and 5
    let replay_res = engine.get_events_since(3);
    assert!(replay_res.is_ok());
    let replayed = replay_res.unwrap();
    assert_eq!(replayed.len(), 2);
    assert_eq!(replayed[0].sequence, 4);
    assert_eq!(replayed[1].sequence, 5);

    // Test 2: Replay from sequence 0 (older than retained buffer) -> should return GAP
    let gap_res = engine.get_events_since(0);
    assert!(gap_res.is_err());
    let oldest = gap_res.unwrap_err();
    assert_eq!(oldest, 2);
}

#[test]
fn test_bank_and_update_parent_lifecycle() {
    let mut config = ServerConfig::default();
    config.source = "local-geyser".to_string();
    let mut engine = CoreStateEngine::new(config);

    let provider = ProviderId::new("local-validator-geyser");
    let slot = Slot(500);

    // 1. Bank A created (bank_id = 1)
    let ident_a = BankIdentity::new(provider.clone(), Some(BankId(1)), slot, Some(Blockhash::new("BlockHash_A")));
    let ev_bank_a = ChronoEvent::new(
        1,
        provider.clone(),
        slot,
        Some(Blockhash::new("BlockHash_A")),
        ChronoEventKind::BankCreated {
            identity: ident_a.clone(),
            parent: ParentReference::SlotOnly(Slot(499)),
        },
    );
    engine.process_event(ev_bank_a);

    // 2. UpdateParent invalidates bank_id = 1
    let ev_update_parent = ChronoEvent::new(
        2,
        provider.clone(),
        slot,
        None,
        ChronoEventKind::UpdateParent {
            slot,
            cleared_bank_id: Some(BankId(1)),
            parent_slot: Slot(499),
            parent_block_id: Some("AuthoritativeParentHash".to_string()),
            fec_set_index: Some(0),
            source: "simd0337_handover".to_string(),
        },
    );
    let s_up = engine.process_event(ev_update_parent);
    assert_eq!(s_up.event_type, "UpdateParent");
    assert_eq!(s_up.bank_id, Some(1));

    // 3. Bank B replacement created (bank_id = 2)
    let ident_b = BankIdentity::new(provider.clone(), Some(BankId(2)), slot, Some(Blockhash::new("BlockHash_B")));
    let ev_bank_b = ChronoEvent::new(
        3,
        provider.clone(),
        slot,
        Some(Blockhash::new("BlockHash_B")),
        ChronoEventKind::BankCreated {
            identity: ident_b.clone(),
            parent: ParentReference::KnownBlockhash(Blockhash::new("AuthoritativeParentHash")),
        },
    );
    engine.process_event(ev_bank_b);

    // 4. Replacement correlation
    let ev_replacement = ChronoEvent::new(
        4,
        provider.clone(),
        slot,
        None,
        ChronoEventKind::ReplacementBank {
            old_identity: ident_a,
            replacement_identity: ident_b.clone(),
            slot,
        },
    );
    engine.process_event(ev_replacement);

    // 5. Canonical resolution for Bank B
    let ev_canonical = ChronoEvent::new(
        5,
        provider.clone(),
        slot,
        Some(Blockhash::new("BlockHash_B")),
        ChronoEventKind::CanonicalObserved {
            canonical_identity: ident_b,
        },
    );
    let s_can = engine.process_event(ev_canonical);
    assert_eq!(s_can.event_type, "CanonicalChanged");

    // 6. Snapshot reflects UpdateParent and banks
    let snapshot = engine.build_snapshot(0);
    assert_eq!(snapshot.parent.total_update_parents, 1);
    let last_up = snapshot.parent.last_update_parent.unwrap();
    assert_eq!(last_up.slot, 500);
    assert_eq!(last_up.cleared_bank_id, Some(1));
    assert_eq!(last_up.replacement_bank_id, Some(2));
}

#[test]
fn test_certificate_integration_and_provenance() {
    let mut config = ServerConfig::default();
    config.source = "local-geyser".to_string();
    let mut engine = CoreStateEngine::new(config);

    let provider = ProviderId::new("local-validator-geyser");
    let slot = Slot(1000);

    let ev_cert = ChronoEvent::new(
        1,
        provider.clone(),
        slot,
        None,
        ChronoEventKind::CertificateObserved {
            slot,
            kind: CertificateKind::FinalCert,
            raw_len: 192,
            raw_bytes: Some(vec![0x7A; 192]),
            block_id: Some("blk-1000".to_string()),
            validation_status: Some(chrono_core::types::CertificateValidationStatus::ParsedUnverified),
            stake_percent: Some(88.4),
        },
    );
    let s_cert = engine.process_event(ev_cert);

    assert_eq!(s_cert.event_type, "CertificateObserved");
    assert_eq!(s_cert.provenance, EventProvenance::DERIVED); // Fixture provenance
    assert_eq!(s_cert.environment, "fixture");

    let snapshot = engine.build_snapshot(1);
    assert_eq!(snapshot.capabilities.coverage_score, 100);
    assert_eq!(snapshot.finality.cert_type, Some("FinalCert".to_string()));
    assert_eq!(snapshot.finality.stake_percent, Some(88.4));
}

#[test]
fn test_public_rpc_limitations_honesty() {
    let mut config = ServerConfig::default();
    config.source = "rpc".to_string();
    config.cluster = "testnet".to_string();
    let mut engine = CoreStateEngine::new(config);

    let provider = ProviderId::new("testnet-rpc");
    let slot = Slot(2000);

    let ev_slot = ChronoEvent::new(
        1,
        provider.clone(),
        slot,
        None,
        ChronoEventKind::SlotObserved { parent_slot: Some(Slot(1999)) },
    );
    let s_slot = engine.process_event(ev_slot);

    assert_eq!(s_slot.provenance, EventProvenance::DIRECT);
    assert_eq!(s_slot.environment, "live");

    let snapshot = engine.build_snapshot(0);
    assert_eq!(snapshot.capabilities.coverage_score, 27);
    assert!(snapshot.capabilities.limitations.len() >= 3);
    assert_eq!(snapshot.finality.mode, "TOWER_BFT_FALLBACK");
    assert_eq!(snapshot.finality.cert_type, None);
}

#[tokio::test]
async fn test_chrono_server_router_endpoints() {
    let mut config = ServerConfig::default();
    config.source = "local-geyser".to_string();
    let server = ChronoServer::new(config);
    let router = server.router();

    use axum::http::Request;
    use tower::ServiceExt;

    // Test GET /api/v1/health
    let req_health = Request::builder()
        .uri("/api/v1/health")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp = router.clone().oneshot(req_health).await.unwrap();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);

    // Test GET /api/v1/snapshot
    let req_snap = Request::builder()
        .uri("/api/v1/snapshot")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp_snap = router.clone().oneshot(req_snap).await.unwrap();
    assert_eq!(resp_snap.status(), axum::http::StatusCode::OK);

    // Test GET /api/v1/capabilities
    let req_caps = Request::builder()
        .uri("/api/v1/capabilities")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp_caps = router.clone().oneshot(req_caps).await.unwrap();
    assert_eq!(resp_caps.status(), axum::http::StatusCode::OK);

    // Test GET /api/v1/transaction/:signature
    let req_tx = Request::builder()
        .uri("/api/v1/transaction/5rJk3F6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a")
        .body(axum::body::Body::empty())
        .unwrap();
    let resp_tx = router.oneshot(req_tx).await.unwrap();
    assert_eq!(resp_tx.status(), axum::http::StatusCode::OK);
}
