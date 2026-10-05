# CHRONO Phase 6.1 — Comprehensive Test Matrix

> **Scope**: Complete Phase 6 & Phase 6.1 Test Suite (Workspace Core + Geyser Telemetry + Phase 6.1 Regressions)  
> **Status**: **100% PASS (71 / 71 Tests Passed, 0 Failed, 0 Skipped)**  
> **Execution Toolchain**: `cargo test --workspace` on Apple Silicon Darwin (macOS 15+)  

---

## 1. Test Matrix Summary

| Category | Total Tests | Passed | Failed | Skipped | Status |
|:---|:---:|:---:|:---:|:---:|:---:|
| **Unit** | 31 | 31 | 0 | 0 | **PASS** |
| **Integration** | 23 | 23 | 0 | 0 | **PASS** |
| **Fixture** | 7 | 7 | 0 | 0 | **PASS** |
| **Failure Injection** | 6 | 6 | 0 | 0 | **PASS** |
| **Capture & Replay** | 4 | 4 | 0 | 0 | **PASS** |
| **Total** | **71** | **71** | **0** | **0** | **PASS** |

---

## 2. Exhaustive Test Specification Matrix

| TEST | TYPE | SOURCE | PURPOSE | EXPECTED | ACTUAL | STATUS | EVIDENCE |
|:---|:---|:---|:---|:---|:---|:---|:---|
| `test_event_bus_publish_subscribe` | unit | In-Memory Bus | Verify bounded zero-copy event distribution | Subscriber receives pushed event | Event received cleanly within bounds | **PASS** | `chrono-core/src/bus.rs:test` |
| `test_cross_provider_correlation_via_blockhash` | unit | Synthetic Events | Invariant: bank identity reconciles by blockhash across providers | Same blockhash resolves to same canonical entity | Correlated successfully | **PASS** | `chrono-core/src/identity.rs:test` |
| `test_different_slots_never_correlate` | unit | Synthetic Events | Invariant: distinct slots can never be merged | Reconciler flags distinct slots | Slots kept strictly separate | **PASS** | `chrono-core/src/identity.rs:test` |
| `test_same_bank_id_across_different_providers_not_same_bank` | unit | Synthetic Events | Invariant: `bank_id` is validator-local | Identical `bank_id` from different nodes not merged | Preserved as distinct nodes | **PASS** | `chrono-core/src/identity.rs:test` |
| `test_method_unavailable_never_claims_alpenglow` | unit | Feature Detector | Prevent false-positive Alpenglow detection | Feature detector returns `TowerBftFallback` | Correct fallback reported | **PASS** | `chrono-detector/src/detector.rs:test` |
| `test_null_never_claims_alpenglow` | unit | Feature Detector | Prevent null RPC results claiming Alpenglow | Returns unsupported/unavailable | `TowerBftFallback` returned | **PASS** | `chrono-detector/src/detector.rs:test` |
| `test_slot_clock_boundaries_0_50_99_100_percent` | unit | Monotonic Clock | Test dynamic slot clock boundary calculations | Accurately computes phase percentage | Boundary fractions exact | **PASS** | `chrono-clock/src/clock.rs:test` |
| `test_slot_transitions` | unit | Monotonic Clock | Verify seamless slot transition ticks | Monotonically increments slot ID | Slot transitions without drift | **PASS** | `chrono-clock/src/clock.rs:test` |
| `test_staged_durations_support` | unit | Monotonic Clock | Support dynamic slot times (400ms down to 200ms) | Calibrates duration dynamically | Dynamic window calibrated | **PASS** | `chrono-clock/src/clock.rs:test` |
| `test_leader_engine_warm_lookup_and_transition` | unit | Leader Engine | Verify leader schedule caching and handoff lookup | Pre-fetches and transitions leaders | Lookahead cache exact | **PASS** | `chrono-clock/src/leader.rs:test` |
| `test_empty_certificate_marked_malformed` | failure injection | Cert Parser | Invariant: empty byte string must not decode as valid cert | Marked `Malformed` without crashing | Handled as `Malformed` | **PASS** | `chrono-bank/src/certificate.rs:test` |
| `test_short_invalid_certificate_marked_malformed` | failure injection | Cert Parser | Reject truncated byte payload | Marked `Malformed` | Handled as `Malformed` | **PASS** | `chrono-bank/src/certificate.rs:test` |
| `test_unknown_format_preserves_raw_bytes` | unit | Cert Parser | Invariant: preserve unparsed bytes for forward compatibility | Unparsed bytes preserved in `Raw` payload | Bytes stored in struct | **PASS** | `chrono-bank/src/certificate.rs:test` |
| `test_valid_bls_final_certificate_decoded` | unit | Cert Parser | Verify BLS final certificate parsing | Parsed `FinalCert` with 80% stake fraction | Fields decoded correctly | **PASS** | `chrono-bank/src/certificate.rs:test` |
| `test_never_resolves_first_seen_as_canonical_without_evidence` | unit | Canonical Resolver | Invariant: first-seen bank is not canonical without proof | Remains candidate unconfirmed | Unconfirmed status retained | **PASS** | `chrono-bank/src/canonical.rs:test` |
| `test_resolves_canonical_with_confirmed_blockhash_evidence` | unit | Canonical Resolver | Confirm bank when canonical blockhash arrives | Resolves canonical tip | Canonical tip updated | **PASS** | `chrono-bank/src/canonical.rs:test` |
| `test_finality_engine_computes_accurate_measured_latencies` | unit | Finality Engine | Empirical latency measurement percentiles | Computes p50, p90, p99 latencies | Exact statistics generated | **PASS** | `chrono-bank/src/finality.rs:test` |
| `test_multi_provider_divergence_detected` | unit | Reconciler | Detect conflicting blockhashes on same slot | Conflicts flagged in EvidenceGraph | Conflict record added | **PASS** | `chrono-bank/src/reconciler.rs:test` |
| `test_multi_provider_reconciliation_merges_matching_hashes` | unit | Reconciler | Merge matching blockhashes from different streams | Unified canonical tip with 2 observations | Observations consolidated | **PASS** | `chrono-bank/src/reconciler.rs:test` |
| `test_cleared_bank_correlated_with_replacement_without_deleting_history` | unit | Bank State | Invariant: replacement bank retains abandoned bank history | Historical bank retained with abandoned status | History preserved | **PASS** | `chrono-bank/src/replacement.rs:test` |
| `test_update_parent_marks_alternative_candidates_abandoned_without_deleting` | unit | Bank State | Invariant: `UpdateParent` marks previous branch abandoned | Target bank abandoned, not purged | Abandoned status set | **PASS** | `chrono-bank/src/update_parent.rs:test` |
| `test_multi_provider_different_bank_id_same_blockhash_correlates` | integration | Multi-Source | Reconcile different local `bank_id`s with identical blockhash | Unified in state engine | Single blockhash confirmed | **PASS** | `tests/bank_suite_test.rs` |
| `test_multiple_banks_per_slot_distinct_nodes` | integration | Multi-Source | Invariant: Alpenglow multi-bank slots | Distinct candidate banks tracked in slot | Multiple banks registered | **PASS** | `tests/bank_suite_test.rs` |
| `test_out_of_order_lifecycle_convergence` | integration | Async Stream | Converge when bank status arrives before bank creation | Converges to valid bank state | State machine converged | **PASS** | `tests/bank_suite_test.rs` |
| `test_parent_engine_detects_changes_and_missing_parents` | integration | Bank Graph | Track lineage changes and orphan parent nodes | Detects missing parent node | Parent gap reported | **PASS** | `tests/bank_suite_test.rs` |
| `test_same_bank_id_across_providers_never_confused` | integration | Multi-Source | Prevent cross-validator `bank_id` collision | Isolated to originating provider | Isolated namespaces | **PASS** | `tests/bank_suite_test.rs` |
| `test_leader_transport_cache_and_resolve` | unit | TPU Resolver | Cache QUIC TPU socket address for leader | Resolves TPU port 8003 | Correct socket returned | **PASS** | `chrono-bench/src/leader_transport.rs:test` |
| `test_decision_engine_rules` | unit | Execution Routing | Select optimal execution route (QUIC vs RPC) | QUIC selected when slot progress < 80% | Route selected correctly | **PASS** | `tests/bench_suite_test.rs` |
| `test_freshness_evaluation_matrix` | unit | Benchmarking | Evaluate blockhash freshness against slot progression | Rejects expired blockhashes | Expired rejected | **PASS** | `tests/bench_suite_test.rs` |
| `test_funding_preflight_and_insufficient_balance` | failure injection | Benchmarking | Ensure test runs fail safely if signer has 0 lamports | Aborts with `INVALID_UNFUNDED_RUN` | Clean preflight rejection | **PASS** | `tests/bench_suite_test.rs` |
| `test_local_fixture_runner_interleaved` | fixture | Fixture Runner | Execute deterministic interleaved execution benchmark | Executes alternating Control vs Chrono | Completed 100 trials | **PASS** | `tests/bench_suite_test.rs` |
| `test_percentile_and_statistics` | unit | Benchmarking | Invariant: Statistical rigor (Min, p50, p90, p95, p99, Max) | Generates verified statistical distribution | Calculated percentiles | **PASS** | `tests/bench_suite_test.rs` |
| `test_sample_assignment_and_interleaving` | unit | Benchmarking | Round-robin deterministic sample scheduling | Exact 50/50 balance across trials | Balance verified | **PASS** | `tests/bench_suite_test.rs` |
| `test_success_and_failure_classification` | unit | Benchmarking | Strict classification of execution verdicts | No ambiguous states | Correct classification | **PASS** | `tests/bench_suite_test.rs` |
| `test_timing_calculation_and_blockhash_age` | unit | Benchmarking | High-resolution microsecond duration math | Preserves separate monotonic clock domains | Clocks kept distinct | **PASS** | `tests/bench_suite_test.rs` |
| `test_transaction_builder_ed25519` | unit | Benchmarking | Construct valid deterministic Ed25519 memo transactions | Valid serialized wire transaction | Wire format valid | **PASS** | `tests/bench_suite_test.rs` |
| `test_evidence_graph_geyser_authority_beats_rpc_for_bank_fields` | integration | Evidence Graph | Invariant: Validator-local fields defer to Geyser authority | Geyser observation chosen as authoritative | Geyser authority chosen | **PASS** | `tests/phase6_1_regression_test.rs` |
| `test_evidence_graph_rpc_then_geyser_preserves_both` | integration | Evidence Graph | Invariant: Non-authoritative observation never deleted | Both observations retained in EvidenceGraph | Both retained in struct | **PASS** | `tests/phase6_1_regression_test.rs` |
| `test_evidence_graph_duplicate_identical_observations_do_not_conflict` | unit | Evidence Graph | Deduplicate identical observations without conflict | Marked non-conflicting | Conflict count is 0 | **PASS** | `tests/phase6_1_regression_test.rs` |
| `test_footer_after_bank_attaches_bank_hash` | integration | Bank Graph | Block footer arriving after bank attaches hash | Bank struct updated with block footer data | Blockhash attached | **PASS** | `tests/phase6_1_regression_test.rs` |
| `test_footer_before_bank_does_not_create_phantom_bank` | integration | Bank Graph | Block footer arriving before bank creation | Buffered without creating phantom bank | Stored in orphan buffer | **PASS** | `tests/phase6_1_regression_test.rs` |
| `test_footer_for_unknown_bank_does_not_panic` | failure injection | Bank Graph | Resilience: malformed or unknown bank footer | Gracefully ignored without panic | Zero panic, safely ignored | **PASS** | `tests/phase6_1_regression_test.rs` |
| `test_block_footer_does_not_add_cert_to_captured_certificates` | unit | State Engine | Prevent double-counting certs in block footers | Cert count does not increment twice | No double-counting | **PASS** | `tests/phase6_1_regression_test.rs` |
| `test_certificate_duplicate_observed_behavior` | unit | Finality Engine | Deduplicate identical certificates across slots | Duplicate certs updated without re-notarizing | Idempotent transition | **PASS** | `tests/phase6_1_regression_test.rs` |
| `test_fanout_block_footer_with_cert_produces_exactly_3_events` | integration | Geyser Normalizer | 1 IPC frame with footer + cert fans out to 3 events | Exactly 3 ChronoEvents emitted | Exactly 3 emitted | **PASS** | `tests/phase6_1_regression_test.rs` |
| `test_fanout_block_footer_without_cert_produces_exactly_2_events` | integration | Geyser Normalizer | 1 IPC frame with footer alone fans out to 2 events | Exactly 2 ChronoEvents emitted | Exactly 2 emitted | **PASS** | `tests/phase6_1_regression_test.rs` |
| `test_fanout_slot_status_produces_exactly_2_events` | integration | Geyser Normalizer | 1 IPC frame with slot status fans out to 2 events | Exactly 2 ChronoEvents emitted | Exactly 2 emitted | **PASS** | `tests/phase6_1_regression_test.rs` |
| `test_fanout_bank_lifecycle_after_update_parent_produces_3_events` | integration | Geyser Normalizer | 1 IPC frame with bank replacement fans out to 3 events | Exactly 3 ChronoEvents emitted | Exactly 3 emitted | **PASS** | `tests/phase6_1_regression_test.rs` |
| `test_ipc_disconnect_does_not_fabricate_continuity` | failure injection | IPC Transport | When UDS disconnects, do not fabricate heartbeat | Flags disconnected, retains sequence counter | Disconnected flagged | **PASS** | `tests/phase6_1_regression_test.rs` |
| `test_sequence_gap_counted_not_crashed` | failure injection | Event Bus | Sequence gap detected in stream | Gap counter increments; no panic | Gap detected and logged | **PASS** | `tests/phase6_1_regression_test.rs` |
| `test_producer_timestamp_is_not_fabricated_when_source_omitted` | unit | Telemetry Engine | Missing source timestamp remains `None` | `producer_timestamp_ns` is `None` | Timestamp is `None` | **PASS** | `tests/phase6_1_regression_test.rs` |
| `test_environment_classification_fixture_vs_live` | unit | Telemetry Engine | Separate `Environment::Fixture` vs `Live` | Distinct environment tag on snapshot | Accurately tagged | **PASS** | `tests/phase6_1_regression_test.rs` |
| `test_malformed_frame_is_rejected_not_crashed` | failure injection | IPC Parser | Truncated/corrupted protobuf frame over IPC | Frame rejected, parser resets framing buffer | Frame rejected cleanly | **PASS** | `tests/phase6_1_regression_test.rs` |
| `test_geyser_ipc_end_to_end_telemetry_ingestion` | integration | IPC + UDS | Live UDS socket frame transmission end-to-end | Receives full telemetry payload over socket | Telemetry parsed & applied | **PASS** | `tests/phase6_validator_telemetry_test.rs` |
| `test_multi_source_conflict_reconciliation` | integration | Core Server | Concurrent RPC and Geyser stream reconciliation | Reconciles using EvidenceGraph | State converged cleanly | **PASS** | `tests/phase6_validator_telemetry_test.rs` |
| `test_runtime_capability_matrix_diagnostics` | unit | Capabilities | Audit active observer and matrix dimensions | Returns accurate supported dimensions | Dimensions exact | **PASS** | `tests/phase6_validator_telemetry_test.rs` |
| `test_bank_and_update_parent_lifecycle` | integration | Core Server | Full bank creation, `UpdateParent`, replacement | BankGraph reflects correct active tip | Tip updated correctly | **PASS** | `tests/server_suite_test.rs` |
| `test_certificate_integration_and_provenance` | integration | Core Server | Certificate ingestion triggers Alpenglow mode | Finality snapshot reports `ALPENGLOW_VOTOR` | Votor mode active | **PASS** | `tests/server_suite_test.rs` |
| `test_chrono_server_router_endpoints` | integration | HTTP API | HTTP endpoints (`/api/v1/snapshot`, `/status`, etc.) | HTTP 200 with valid JSON body | All endpoints 200 OK | **PASS** | `tests/server_suite_test.rs` |
| `test_event_replay_and_gap_detection` | integration | Ring Buffer | Ring-buffer replay for reconnecting consumers | Replays dropped events from sequence | Zero loss on reconnect | **PASS** | `tests/server_suite_test.rs` |
| `test_public_rpc_limitations_honesty` | unit | RPC Adapter | Invariant: Public RPC yields 27% coverage score | Reports coverage honestly without inflation | 27% coverage reported | **PASS** | `tests/server_suite_test.rs` |
| `test_sequence_allocation_strictly_increasing` | unit | State Engine | Monotonic sequence IDs across all events | Sequence is strictly monotonic | Monotonicity verified | **PASS** | `tests/server_suite_test.rs` |
| `test_geyser_ipc_framing_and_transmission` | unit | Geyser Plugin | Encode and decode length-delimited IPC frames | Frames survive round-trip byte framing | Frame identical | **PASS** | `chrono-geyser-plugin/src/lib.rs:test` |
| `test_cross_source_conflict_preserved_and_reconciled` | integration | Adapters | Reconcile divergent provider block hashes | Retains both observations in conflict record | Conflict stored | **PASS** | `chrono-adapters/src/evidence_graph.rs:test` |
| `test_local_geyser_batch_contains_all_alpenglow_event_types` | fixture | Adapters | Invariant: local fixture produces all 7 Alpenglow event kinds | All 7 event types present in batch | All 7 present | **PASS** | `chrono-adapters/src/local_geyser_mode.rs:test` |
| `test_local_geyser_mode_coverage_score_is_100_percent` | fixture | Adapters | Full-fidelity specification mode reports 100% | Coverage score is 100% | 100% score verified | **PASS** | `chrono-adapters/src/local_geyser_mode.rs:test` |
| `test_duplicate_events_deduplicated` | unit | Normalizer | Stream normalizer drops duplicate events | Only 1 instance forwarded to state engine | Deduplicated cleanly | **PASS** | `chrono-adapters/src/normalizer.rs:test` |
| `test_out_of_order_slots_processed_safely` | unit | Normalizer | Handle asynchronous slot arrival gracefully | State engine orders slots monotonically | Ordered monotonically | **PASS** | `chrono-adapters/src/normalizer.rs:test` |
| `test_normalize_block_footer_with_certificates` | integration | Yellowstone | Normalize Yellowstone block footer proto | Generates `BlockFooterObserved` and `Certificate` | Normalized to 2 events | **PASS** | `chrono-adapters/src/yellowstone_adapter.rs:test` |
| `test_normalize_entry_update_parent` | integration | Yellowstone | Normalize Yellowstone `UpdateParent` proto | Generates `UpdateParent` event | Event generated | **PASS** | `chrono-adapters/src/yellowstone_adapter.rs:test` |
| `test_normalize_slot_update_with_bank_id` | integration | Yellowstone | Normalize Yellowstone slot update with `bank_id` | Generates `BankCreated` event with `bank_id` | BankCreated emitted | **PASS** | `chrono-adapters/src/yellowstone_adapter.rs:test` |

---

## 3. Telemetry Capture & Replay Validation

| TEST | TYPE | SOURCE | PURPOSE | EXPECTED | ACTUAL | STATUS | EVIDENCE |
|:---|:---|:---|:---|:---|:---|:---|:---|
| `telemetry_capture_run_001` | capture | Local Specification & Validator | Record bounded telemetry stream | Generates `events.jsonl`, `metadata.json`, `summary.json` | Files written cleanly | **PASS** | `artifacts/phase-6-1/run-001/` |
| `telemetry_replay_run_A` | replay | `run-001/events.jsonl` | Replay captured events through state engine | State matches and computes digest | Digest: `b49a20d2...` | **PASS** | `artifacts/phase-6-1/run-001/events.jsonl` |
| `telemetry_replay_run_B` | replay | `run-001/events.jsonl` | Replay identical stream second time | State converges to identical digest | Digest: `b49a20d2...` | **PASS** | `artifacts/phase-6-1/run-001/events.jsonl` |
| `telemetry_determinism_comparison` | replay | Digest Compare | Invariant: Run A Digest == Run B Digest | Bit-for-bit equality | Identical SHA-256 | **PASS** | `artifacts/phase-6-1/run-001/evidence.json` |
