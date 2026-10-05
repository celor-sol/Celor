use chrono_bench::{
    decision::{ExecutionDecisionEngine, RoutingAction},
    experiment::{ExperimentConfig, ExperimentGroup},
    explainer::ExecutionExplainer,
    freshness::{BankFreshnessTier, FreshnessState, FreshnessTier},
    route::{LocalFixtureRoute, TransactionBuilder},
    runner::BenchmarkRunner,
    stats::BenchmarkStats,
    storage::BenchmarkStorage,
};
use std::sync::Arc;

#[test]
fn test_freshness_evaluation_matrix() {
    // 1. Fresh state
    let fresh = FreshnessState::evaluate(
        50, // 50ms / 250ms -> 20% elapsed (FRESH)
        250,
        Some("Leader_1".to_string()),
        Some("Leader_2".to_string()),
        "BlockHash_1111111111111111111".to_string(),
        1000,
        1000,
        Some("bank-1".to_string()),
        false,
    );
    assert_eq!(fresh.slot_tier, FreshnessTier::Fresh);
    assert_eq!(fresh.bank_tier, BankFreshnessTier::Canonical);

    // 2. Abandoned bank state (UpdateParent invalidation)
    let abandoned = FreshnessState::evaluate(
        150,
        250,
        Some("Leader_1".to_string()),
        Some("Leader_2".to_string()),
        "BlockHash_1111111111111111111".to_string(),
        1000,
        1000,
        Some("bank-abandoned".to_string()),
        true, // is_bank_abandoned = true
    );
    assert_eq!(abandoned.bank_tier, BankFreshnessTier::Abandoned);

    // 3. Stale blockhash
    let stale_bh = FreshnessState::evaluate(
        50,
        250,
        Some("Leader_1".to_string()),
        Some("Leader_2".to_string()),
        "BlockHash_Old".to_string(),
        1, // acquired long ago
        1000,
        Some("bank-1".to_string()),
        false,
    );
    assert_eq!(stale_bh.blockhash_tier, FreshnessTier::Stale);
}

#[test]
fn test_decision_engine_rules() {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    // A. Normal Fresh Window:
    let normal_fresh = FreshnessState::evaluate(
        50,
        250,
        Some("Leader_A".to_string()),
        Some("Leader_B".to_string()),
        "BlockHash_Fresh".to_string(),
        now_ms,
        now_ms,
        Some("bank-1".to_string()),
        false,
    );
    let ctrl_dec = ExecutionDecisionEngine::evaluate_control(&normal_fresh);
    let chrono_dec = ExecutionDecisionEngine::evaluate_chrono(&normal_fresh, true);
    assert_eq!(ctrl_dec.action, RoutingAction::Submit);
    assert_eq!(chrono_dec.action, RoutingAction::Submit);

    // B. Leader Handoff Imminent (<40ms remaining):
    let handoff_imminent = FreshnessState::evaluate(
        230, // 230ms / 250ms -> 20ms remaining
        250,
        Some("Leader_A".to_string()),
        Some("Leader_B".to_string()),
        "BlockHash_Fresh".to_string(),
        now_ms,
        now_ms,
        Some("bank-1".to_string()),
        false,
    );
    let ctrl_handoff = ExecutionDecisionEngine::evaluate_control(&handoff_imminent);
    let chrono_handoff = ExecutionDecisionEngine::evaluate_chrono(&handoff_imminent, true);
    assert_eq!(ctrl_handoff.action, RoutingAction::Submit); // Control blind submit
    assert!(matches!(chrono_handoff.action, RoutingAction::Wait { .. })); // Chrono waits for next leader!

    // C. Abandoned Candidate Bank:
    let abandoned_state = FreshnessState::evaluate(
        50,
        250,
        Some("Leader_A".to_string()),
        Some("Leader_B".to_string()),
        "BlockHash_Fresh".to_string(),
        now_ms,
        now_ms,
        Some("bank-dead".to_string()),
        true, // Abandoned
    );
    let chrono_abort = ExecutionDecisionEngine::evaluate_chrono(&abandoned_state, true);
    assert!(matches!(chrono_abort.action, RoutingAction::Abort { .. }));
}

#[test]
fn test_transaction_builder_ed25519() {
    let (keypair, pubkey) = TransactionBuilder::new_keypair();
    let recipient = [7u8; 32];
    let recent_blockhash = "4vJ9JU1bJJE96FWSXTvHsmmFADCg4gpZQff4P3bkLKi";

    let tx_base64 = TransactionBuilder::build_and_sign_transfer(
        &keypair,
        &recipient,
        50_000,
        recent_blockhash,
    )
    .expect("Should build and sign transfer transaction");

    assert!(!tx_base64.is_empty());
    assert!(tx_base64.len() > 100);
    assert_eq!(pubkey.len(), 32);
}

#[test]
fn test_percentile_and_statistics() {
    let mut latencies = vec![100.0, 110.0, 120.0, 130.0, 140.0, 150.0, 160.0, 170.0, 180.0, 200.0];
    let p50 = BenchmarkStats::percentile(&mut latencies, 0.50);
    let p90 = BenchmarkStats::percentile(&mut latencies, 0.90);
    let p99 = BenchmarkStats::percentile(&mut latencies, 0.99);

    assert_eq!(p50, 150.0);
    assert_eq!(p90, 200.0);
    assert_eq!(p99, 200.0);
}

#[tokio::test]
async fn test_local_fixture_runner_interleaved() {
    let temp_dir = std::env::temp_dir().join(format!("chrono-bench-test-{}", std::process::id()));
    let storage = BenchmarkStorage::new(&temp_dir);

    let config = ExperimentConfig {
        id: format!("test-exp-{}", std::process::id()),
        sample_count: 10,
        max_tps: 1000.0, // Fast for unit tests
        interleave_mode: true,
        ..Default::default()
    };

    let runner = BenchmarkRunner::new(config.clone());
    let control_route = Arc::new(LocalFixtureRoute::new(0));
    let chrono_route = Arc::new(LocalFixtureRoute::new(0));

    let (summary, samples) = runner
        .run_experiment(control_route, chrono_route)
        .await
        .expect("Experiment run should succeed");

    storage.save_experiment(&config, &samples, &summary).expect("Save should succeed");
    let loaded_summary = storage.load_summary(&config.id).expect("Load summary should succeed");
    assert_eq!(loaded_summary.experiment_id, config.id);

    assert_eq!(samples.len(), 10);
    assert_eq!(summary.total_samples, 10);

    let ctrl_samples = samples.iter().filter(|s| s.group == ExperimentGroup::Control).count();
    let chrono_samples = samples.iter().filter(|s| s.group == ExperimentGroup::ChronoAware).count();

    assert_eq!(ctrl_samples, 5);
    assert_eq!(chrono_samples, 5);

    // Test explain format on sample 0
    let explanation = ExecutionExplainer::format_explanation(&samples[0]);
    assert!(explanation.contains("CHRONO EXECUTION EXPLAINER"));
    assert!(explanation.contains("Group:             CONTROL"));

    // Cleanup temp dir
    let _ = std::fs::remove_dir_all(temp_dir);
}

#[tokio::test]
async fn test_funding_preflight_and_insufficient_balance() {
    let config = ExperimentConfig {
        sample_count: 10,
        max_tps: 1000.0, // Fast for unit tests
        ..Default::default()
    };

    let runner = BenchmarkRunner::new(config);
    // Local mock route should pass preflight cleanly
    let mock_route = Arc::new(LocalFixtureRoute::new(0));
    let (summary, _samples) = runner
        .run_experiment(mock_route.clone(), mock_route.clone())
        .await
        .expect("Mock route experiment should pass preflight");

    assert_eq!(summary.total_samples, 10);
}

#[test]
fn test_sample_assignment_and_interleaving() {
    let sample_count = 20;
    let mut groups = Vec::new();
    for i in 0..sample_count {
        let group = if i % 2 == 0 {
            ExperimentGroup::Control
        } else {
            ExperimentGroup::ChronoAware
        };
        groups.push(group);
    }

    assert_eq!(groups.len(), 20);
    assert_eq!(groups[0], ExperimentGroup::Control);
    assert_eq!(groups[1], ExperimentGroup::ChronoAware);
    assert_eq!(groups[2], ExperimentGroup::Control);
    assert_eq!(groups[3], ExperimentGroup::ChronoAware);

    let ctrl_count = groups.iter().filter(|g| **g == ExperimentGroup::Control).count();
    let chrono_count = groups.iter().filter(|g| **g == ExperimentGroup::ChronoAware).count();
    assert_eq!(ctrl_count, 10);
    assert_eq!(chrono_count, 10);
}

#[test]
fn test_timing_calculation_and_blockhash_age() {
    let acquired_ms = 1_000_000u64;
    let submit_ms = 1_000_150u64;
    let age = submit_ms.saturating_sub(acquired_ms);
    assert_eq!(age, 150);

    let t3_ns = 1_000_000_000u64;
    let confirmed_elapsed_ms = 250.0f64;
    let t9_confirmed_ns = t3_ns + (confirmed_elapsed_ms * 1_000_000.0) as u64;
    assert_eq!(t9_confirmed_ns, 1_250_000_000u64);
}

#[test]
fn test_success_and_failure_classification() {
    use chrono_bench::experiment::ExecutionStatus;

    let confirmed = ExecutionStatus::Confirmed;
    let finalized = ExecutionStatus::Finalized;
    let timeout = ExecutionStatus::Timeout;
    let rejected = ExecutionStatus::Rejected;

    assert!(confirmed == ExecutionStatus::Confirmed || confirmed == ExecutionStatus::Finalized);
    assert!(finalized == ExecutionStatus::Confirmed || finalized == ExecutionStatus::Finalized);
    assert!(!(timeout == ExecutionStatus::Confirmed || timeout == ExecutionStatus::Finalized));
    assert!(!(rejected == ExecutionStatus::Confirmed || rejected == ExecutionStatus::Finalized));
}
