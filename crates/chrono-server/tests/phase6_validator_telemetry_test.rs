use chrono_adapters::{
    capabilities::ProviderCapabilityMatrix,
    evidence_graph::SourceEvidenceGraph,
    geyser_uds_adapter::ValidatorGeyserAdapter,
    provider::ProviderAdapter,
};
use chrono_core::bus::EventBus;
use chrono_core::types::{
    CertificateKind, CertificateValidationStatus, FieldProvenance,
    ObserverContext, ProviderId, Slot, TelemetryLevel,
};
use chrono_geyser_plugin::events::{ChronoGeyserRawFrame, GeyserPayload};
use chrono_geyser_plugin::ipc::GeyserIpcWriter;
use chrono_server::{envelope::EventProvenance, state::CoreStateEngine, ServerConfig};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_geyser_ipc_end_to_end_telemetry_ingestion() {
    let socket_path = "/tmp/chrono_phase6_test.sock";
    let _ = std::fs::remove_file(socket_path);

    let event_bus = Arc::new(EventBus::new(1024));
    let mut rx = event_bus.subscribe();

    let mut adapter = ValidatorGeyserAdapter::new(socket_path, Some(event_bus.clone()));
    adapter.connect().await.expect("Failed to bind adapter UDS server");

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Connect writer (simulating validator-side Geyser plugin)
    let writer = GeyserIpcWriter::new(socket_path.to_string(), 256);
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Stream sequence of validator telemetry messages
    let slot = 500_000;
    let make_frame = |seq: u64, payload: GeyserPayload| ChronoGeyserRawFrame {
        sequence: seq,
        observer_validator: "Local Agave Test Node".to_string(),
        observed_at_nanos: 1_700_000_000_000_000_000,
        payload,
    };

    // 1. Slot update
    writer.dispatch(make_frame(1, GeyserPayload::SlotStatus {
        slot,
        parent_slot: Some(slot - 1),
        bank_id: Some(1),
        status: "Processed".to_string(),
    }));

    // 2. Candidate Bank 1 Created
    writer.dispatch(make_frame(2, GeyserPayload::BankLifecycle {
        slot,
        bank_id: 1,
        parent_slot: Some(slot - 1),
        status: "CreatedBank".to_string(),
        bank_hash: None,
    }));

    // 3. Fast leader handover UpdateParent marker received
    writer.dispatch(make_frame(3, GeyserPayload::UpdateParent {
        slot,
        cleared_bank_id: Some(1),
        parent_slot: slot - 2,
        parent_block_id: Some("alt-parent-block".to_string()),
        fec_set_index: Some(0),
        source: "entry".to_string(),
    }));

    // 4. Replacement Candidate Bank 2 Created
    writer.dispatch(make_frame(4, GeyserPayload::BankLifecycle {
        slot,
        bank_id: 2,
        parent_slot: Some(slot - 2),
        status: "CreatedBank".to_string(),
        bank_hash: None,
    }));

    // 5. Pre-execution deshred transaction observed
    writer.dispatch(make_frame(5, GeyserPayload::DeshredTx {
        slot,
        signature: "5wK9JU1bJJE96nDqasdf112233445566778899aabbccddeeff".to_string(),
        raw_bytes: vec![0x01, 0x02, 0x03, 0x04],
        static_accounts: vec!["11111111111111111111111111111111".to_string()],
        pre_execution_timestamp_nanos: 1_700_000_000_000_000_000,
        fec_set_index: Some(0),
    }));

    // 6. Entry stream notification
    writer.dispatch(make_frame(6, GeyserPayload::Entry {
        slot,
        bank_id: Some(2),
        entry_index: 3,
        starting_tx_index: 0,
        tx_count: 128,
    }));

    // 7. Alpenglow Block Footer with Final Certificate and Producer Nanos
    writer.dispatch(make_frame(7, GeyserPayload::BlockFooter {
        slot,
        bank_id: 2,
        bank_hash: "8xMvY3ZqW998877665544332211aabbccddeeff".to_string(),
        producer_time_nanos: 1_700_000_000_150_000_000,
        user_agent: "agave/v2.0.1".to_string(),
        final_cert: Some(vec![0x42; 192]),
        notar_cert: None,
        skip_cert: None,
    }));

    // Receive normalized events in Chrono Core
    let mut config = ServerConfig::default();
    config.source = "local-validator".to_string();
    let mut engine = CoreStateEngine::new(config);

    // Drain all 13 events: SlotStatus→2, BankLifecycle(1)→2, UpdateParent→1,
    // BankLifecycle(2)→3 (BankObserved+BankStatusChanged+ReplacementBank),
    // DeshredTx→1, Entry→1, BlockFooter→3 (footer+timing+cert)
    let mut received_count = 0;
    while received_count < 13 {
        if let Ok(ev) = tokio::time::timeout(Duration::from_millis(500), rx.recv()).await {
            let core_ev = (*ev.unwrap()).clone();
            let svc_ev = engine.process_event(core_ev);
            // All events from a local-validator Geyser source must be DIRECT or DERIVED.
            // UNAVAILABLE would indicate a data availability failure.
            assert_ne!(svc_ev.provenance, EventProvenance::UNAVAILABLE,
                "Event {} must not be UNAVAILABLE from a local-validator source", svc_ev.event_type);
            received_count += 1;
        } else {
            break;
        }
    }

    assert_eq!(received_count, 13, "All 13 normalized events (from 7 Geyser frames) must be received");

    // Brief yield to let the adapter's async task finish updating counters.
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Inspect live counters
    let counters = adapter.counters().snapshot();
    assert_eq!(counters.bank_events, 2);
    assert_eq!(counters.update_parent_events, 1);
    assert_eq!(counters.block_footer_events, 1);
    assert_eq!(counters.deshred_events, 1);
    assert_eq!(counters.entry_events, 1);
    assert_eq!(counters.slot_events, 1);
    assert_eq!(counters.events_dropped_total, 0, "No drops on non-blocking ring buffer");
    assert_eq!(counters.events_parse_failed, 0, "All valid NDJSON frames parsed");
    assert_eq!(counters.events_out_of_order, 0);

    // Verify Chrono Core consensus state machine
    let snapshot = engine.build_snapshot(0);
    assert_eq!(snapshot.slot.current_slot, slot);
    assert_eq!(snapshot.slot.provenance, EventProvenance::DIRECT);

    // Candidate banks: slot root, bank-1 (abandoned by UpdateParent) and bank-2 (active candidate)
    let candidates = snapshot.banks.candidate_banks;
    assert_eq!(candidates.len(), 3, "BankGraph must track slot root and both candidate banks");
    let bank1 = candidates.iter().find(|b| b.raw_bank_id == Some(1)).expect("Bank 1 must exist");
    assert_eq!(bank1.state, "ABANDONED", "Bank 1 must be marked ABANDONED after UpdateParent");

    let bank2 = candidates.iter().find(|b| b.raw_bank_id == Some(2)).expect("Bank 2 must exist");
    assert_eq!(bank2.bank_hash, Some("8xMvY3ZqW998877665544332211aabbccddeeff".to_string()));

    // Verify UpdateParent tracking
    let last_up = snapshot.parent.last_update_parent.expect("UpdateParent summary must be recorded");
    assert_eq!(last_up.slot, slot);
    assert_eq!(last_up.cleared_bank_id, Some(1));
    assert_eq!(last_up.replacement_bank_id, Some(2));
    assert_eq!(last_up.parent_slot, slot - 2);

    // Verify Certificates Subsystem
    let certs = engine.captured_certificates();
    assert_eq!(certs.len(), 1);
    let cert = &certs[0];
    assert_eq!(cert.kind, CertificateKind::FinalCert);
    assert_eq!(cert.slot, Slot(slot));
    assert_eq!(cert.raw_len, 192);
    assert_eq!(cert.validation_status, CertificateValidationStatus::ParsedUnverified);
    assert_eq!(cert.raw_bytes, vec![0x42; 192]);

    // Verify Producer Timing Forensics
    let timing = engine.last_producer_timing().expect("Producer timing must be preserved");
    assert_eq!(timing["producer_time_nanos"], 1_700_000_000_150_000_000u64);

    let _ = adapter.disconnect().await;
    let _ = std::fs::remove_file(socket_path);
}

#[test]
fn test_multi_source_conflict_reconciliation() {
    let mut graph = SourceEvidenceGraph::new();

    let slot = Slot(600_000);
    let source_rpc = ProviderId::new("testnet-rpc");
    let source_yellowstone = ProviderId::new("yellowstone-grpc");
    let source_geyser = ProviderId::new("validator-geyser");

    // Source A (RPC) observes slot at T=100
    graph.record_observation(
        source_rpc.clone(),
        slot,
        None,
        "slot",
        "600000",
        FieldProvenance::Direct,
        100_000_000,
    );

    // Source B (Yellowstone) observes bank_id=10 at T=80
    graph.record_observation(
        source_yellowstone.clone(),
        slot,
        None,
        "bank_id",
        "10",
        FieldProvenance::Direct,
        80_000_000,
    );

    // Source C (Validator Geyser) observes bank_id=10 at T=50 (earliest!)
    graph.record_observation(
        source_geyser.clone(),
        slot,
        None,
        "bank_id",
        "10",
        FieldProvenance::Direct,
        50_000_000,
    );

    let evidence = graph.get_slot_evidence(slot).expect("Slot evidence must exist");
    assert_eq!(evidence.first_observed_by, Some(source_geyser.clone()), "Validator Geyser observed bank_id first");
    assert_eq!(evidence.first_observed_nanos, Some(50_000_000));
    assert!(!evidence.has_conflicts);

    // Introduce conflicting bank hash from external source D
    let source_rogue = ProviderId::new("rogue-provider");
    graph.record_observation(
        source_geyser.clone(),
        slot,
        None,
        "bank_hash",
        "hash_correct_AAAA",
        FieldProvenance::Direct,
        52_000_000,
    );
    graph.record_observation(
        source_rogue.clone(),
        slot,
        None,
        "bank_hash",
        "hash_conflicting_BBBB",
        FieldProvenance::Direct,
        90_000_000,
    );

    let updated_evidence = graph.get_slot_evidence(slot).expect("Slot evidence must exist");
    assert!(updated_evidence.has_conflicts, "Conflict must be detected and flagged");
    assert!(updated_evidence.conflict_summary.as_ref().unwrap().contains("Conflict on field 'bank_hash'"));
    // Ensure all 5 observations are preserved in evidence history without destructive overwriting
    assert_eq!(updated_evidence.observations.len(), 5);
}

#[test]
fn test_runtime_capability_matrix_diagnostics() {
    // 1. Validator Full-Fidelity
    let validator_matrix = ProviderCapabilityMatrix::local_validator("local-validator");
    let det_validator = validator_matrix.to_detailed_matrix(
        ObserverContext {
            observing_validator: Some("Agave 2.0.1 Test Node".into()),
            producing_validator: None,
            transport_type: "uds".into(),
            cluster_environment: "local-validator".into(),
        },
        true,
    );
    assert_eq!(det_validator.telemetry_level, TelemetryLevel::Level4ChronoValidator);
    assert_eq!(det_validator.core_coverage_percent, 100);
    assert_eq!(det_validator.extended_coverage_percent, 100);
    assert!(det_validator.fields.iter().all(|f| f.provenance == FieldProvenance::Direct || f.provenance == FieldProvenance::Derived));

    // 2. Standard Public RPC
    let rpc_matrix = ProviderCapabilityMatrix::standard_public_rpc("testnet-rpc");
    let det_rpc = rpc_matrix.to_detailed_matrix(
        ObserverContext {
            observing_validator: None,
            producing_validator: None,
            transport_type: "rpc".into(),
            cluster_environment: "testnet".into(),
        },
        true,
    );
    assert_eq!(det_rpc.telemetry_level, TelemetryLevel::Level0PublicRpc);
    assert_eq!(det_rpc.core_coverage_percent, 27);
    assert_eq!(det_rpc.extended_coverage_percent, 20);

    // Verify factual reason for unavailable fields
    let bank_id_field = det_rpc.fields.iter().find(|f| f.field == "bank_id").unwrap();
    assert_eq!(bank_id_field.provenance, FieldProvenance::Unavailable);
    assert!(bank_id_field.reason_unavailable.as_ref().unwrap().contains("standard Solana JSON-RPC does not expose bank_id"));
}
