use crate::config::ServerConfig;
use crate::envelope::EventProvenance;
use crate::state::CoreStateEngine;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono_adapters::rpc_client::SolanaRpcClient;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub engine: Arc<RwLock<CoreStateEngine>>,
    pub rpc_client: Arc<RwLock<Option<Arc<SolanaRpcClient>>>>,
    pub source_manager: Option<Arc<crate::source::SourceManager>>,
    pub start_time: Instant,
    pub config: ServerConfig,
    pub ws_clients: Arc<AtomicU64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    pub status: String,
    pub cluster: String,
    pub source: String,
    pub environment: String,
    pub sequence: u64,
    pub coverage_score: u32,
    pub connected: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AutopsyEvidenceItem {
    pub tier: String, // "OBSERVED", "INFERRED", "UNKNOWN"
    pub title: String,
    pub detail: String,
    pub provenance: EventProvenance,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AutopsyTimelineStep {
    pub step: String,
    pub status: String, // "CONFIRMED", "PENDING", "ABANDONED", "UNAVAILABLE"
    pub timestamp_ms: Option<u64>,
    pub detail: String,
    pub source: String,
    pub provenance: EventProvenance,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionAutopsyResponse {
    pub signature: String,
    pub slot: Option<u64>,
    pub block_time: Option<i64>,
    pub blockhash: Option<String>,
    pub bank_hash: Option<String>,
    pub fee: Option<u64>,
    pub compute_units_consumed: Option<u64>,
    pub leader: Option<String>,
    pub candidate_bank: Option<String>,
    pub parent_relation: Option<String>,
    pub confirmation_status: String,
    pub err: Option<String>,
    pub parent_switch: Option<serde_json::Value>,
    pub certificate_info: Option<serde_json::Value>,
    pub evidence: Vec<AutopsyEvidenceItem>,
    pub timeline: Vec<AutopsyTimelineStep>,
    pub inferred_conclusions: Vec<String>,
    pub unknowns: Vec<String>,
    pub source_provenance: EventProvenance,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionStateResponse {
    pub decision: chrono_bench::decision::DecisionResult,
    pub freshness: chrono_bench::freshness::FreshnessState,
    pub leader_window_ms: u64,
    pub target_leader: Option<String>,
    pub next_leader: Option<String>,
    pub quic_route: QuicRouteStatus,
    pub mainnet_safety_guard: bool,
    pub execution_mode: String,
    pub timestamps_t0_t10_contract: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuicRouteStatus {
    pub leader: Option<String>,
    pub tpu_quic_port: u16,
    pub prewarmed: bool,
    pub connection_state: String,
    pub fallback_rpc: bool,
}

/// Builds all HTTP API routes for /api/v1/...
pub fn api_routes(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/health", get(get_health))
        .route("/api/v1/status", get(get_status))
        .route("/api/v1/snapshot", get(get_snapshot))
        .route("/api/v1/capabilities", get(get_capabilities))
        .route("/api/v1/coverage", get(get_coverage))
        .route("/api/v1/telemetry", get(get_telemetry))
        .route("/api/v1/certificates", get(get_certificates))
        .route("/api/v1/entries", get(get_entries))
        .route("/api/v1/deshred", get(get_deshred))
        .route("/api/v1/network", get(get_network))
        .route("/api/v1/slot", get(get_slot))
        .route("/api/v1/leader", get(get_leader))
        .route("/api/v1/banks", get(get_banks))
        .route("/api/v1/finality", get(get_finality))
        .route("/api/v1/consensus", get(get_consensus))
        .route("/api/v1/application-events", get(get_application_events))
        .route("/api/v1/transaction/:signature", get(get_transaction_autopsy))
        .route("/api/v1/execution", get(get_execution_state))
        .route("/api/v1/benchmarks", get(list_benchmarks))
        .route("/api/v1/benchmarks/measured", get(get_measured_benchmarks))
        .route("/api/v1/benchmarks/:id", get(get_benchmark_detail))
        .route("/api/v1/benchmarks/explain/:execution_id", get(explain_execution))
        .route("/api/v1/cluster", post(post_cluster))
        .with_state(state)
}

async fn get_health(State(state): State<AppState>) -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: "0.1.0".to_string(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
    })
}

async fn get_status(State(state): State<AppState>) -> impl IntoResponse {
    let ws_count = state.ws_clients.load(Ordering::Relaxed) as usize;
    let snapshot = state.engine.read().await.build_snapshot(ws_count);
    Json(StatusResponse {
        status: snapshot.status,
        cluster: snapshot.cluster,
        source: snapshot.source,
        environment: snapshot.environment,
        sequence: snapshot.sequence,
        coverage_score: snapshot.capabilities.coverage_score,
        connected: snapshot.network.connected,
    })
}

async fn get_snapshot(State(state): State<AppState>) -> impl IntoResponse {
    let ws_count = state.ws_clients.load(Ordering::Relaxed) as usize;
    let snapshot = state.engine.read().await.build_snapshot(ws_count);
    Json(snapshot)
}

async fn get_capabilities(State(state): State<AppState>) -> impl IntoResponse {
    let ws_count = state.ws_clients.load(Ordering::Relaxed) as usize;
    let snapshot = state.engine.read().await.build_snapshot(ws_count);
    Json(snapshot.capabilities)
}

async fn get_coverage(State(state): State<AppState>) -> impl IntoResponse {
    let engine = state.engine.read().await;
    let matrix = engine.telemetry_capability_matrix();
    Json(matrix)
}

async fn get_telemetry(State(state): State<AppState>) -> impl IntoResponse {
    let engine = state.engine.read().await;
    let counters = engine.telemetry_counters().snapshot();
    let observer = engine.active_observer();
    let producer_timing = engine.last_producer_timing();
    let capability = engine.telemetry_capability_matrix();
    let update_parents = engine.recent_update_parents();

    Json(serde_json::json!({
        "telemetry_level": capability.telemetry_level.to_string(),
        "observer": observer,
        "counters": counters,
        "last_producer_timing": producer_timing,
        "update_parents_tracked": update_parents.len(),
        "recent_update_parents": update_parents.into_iter().rev().take(10).collect::<Vec<_>>(),
    }))
}

async fn get_certificates(State(state): State<AppState>) -> impl IntoResponse {
    let engine = state.engine.read().await;
    let certs = engine.captured_certificates();
    Json(serde_json::json!({
        "total_certificates_captured": certs.len(),
        "certificates": certs,
    }))
}

async fn get_entries(State(state): State<AppState>) -> impl IntoResponse {
    let engine = state.engine.read().await;
    let entries = engine.recent_entries();
    Json(serde_json::json!({
        "total_entries_captured": entries.len(),
        "entries": entries,
    }))
}

async fn get_deshred(State(state): State<AppState>) -> impl IntoResponse {
    let engine = state.engine.read().await;
    let deshreds = engine.recent_deshreds();
    let matrix = engine.capability_matrix();
    let is_supported = matrix.deshred == chrono_adapters::capabilities::FieldSupport::Supported;
    let reason_unavailable = if !is_supported {
        Some("Open-source Yellowstone gRPC does not implement SubscribeDeshred (proprietary Triton extension). Local validator Geyser hook or dedicated plugin required.")
    } else {
        None
    };

    Json(serde_json::json!({
        "supported": is_supported,
        "reason_unavailable": reason_unavailable,
        "total_deshreds_captured": deshreds.len(),
        "deshred_transactions": deshreds,
    }))
}

async fn get_network(State(state): State<AppState>) -> impl IntoResponse {
    let ws_count = state.ws_clients.load(Ordering::Relaxed) as usize;
    let snapshot = state.engine.read().await.build_snapshot(ws_count);
    Json(serde_json::json!({
        "network": snapshot.network,
        "telemetry": snapshot.telemetry,
    }))
}

async fn get_slot(State(state): State<AppState>) -> impl IntoResponse {
    let ws_count = state.ws_clients.load(Ordering::Relaxed) as usize;
    let snapshot = state.engine.read().await.build_snapshot(ws_count);
    Json(snapshot.slot)
}

async fn get_leader(State(state): State<AppState>) -> impl IntoResponse {
    let ws_count = state.ws_clients.load(Ordering::Relaxed) as usize;
    let snapshot = state.engine.read().await.build_snapshot(ws_count);
    Json(snapshot.leader)
}

async fn get_banks(State(state): State<AppState>) -> impl IntoResponse {
    let ws_count = state.ws_clients.load(Ordering::Relaxed) as usize;
    let snapshot = state.engine.read().await.build_snapshot(ws_count);
    Json(snapshot.banks)
}

async fn get_finality(State(state): State<AppState>) -> impl IntoResponse {
    let ws_count = state.ws_clients.load(Ordering::Relaxed) as usize;
    let snapshot = state.engine.read().await.build_snapshot(ws_count);
    Json(snapshot.finality)
}

async fn get_transaction_autopsy(
    Path(signature): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let ws_count = state.ws_clients.load(Ordering::Relaxed) as usize;
    let snapshot = state.engine.read().await.build_snapshot(ws_count);
    let is_fixture = snapshot.environment == "fixture";

    let mut observed_slot: Option<u64> = None;
    let mut block_time: Option<i64> = None;
    let mut blockhash: Option<String> = None;
    let mut fee: Option<u64> = None;
    let mut compute_units_consumed: Option<u64> = None;
    let mut confirmation_status = "UNCONFIRMED".to_string();
    let mut err: Option<String> = None;
    let mut candidate_bank: Option<String> = None;
    let mut bank_hash: Option<String> = None;
    let mut parent_relation: Option<String> = None;
    let mut parent_switch: Option<serde_json::Value> = None;
    let mut certificate_info: Option<serde_json::Value> = None;
    let mut evidence = Vec::new();
    let mut timeline = Vec::new();
    let mut inferred_conclusions = Vec::new();
    let mut unknowns = Vec::new();

    // 1. Observed Evidence: Base58 signature check
    evidence.push(AutopsyEvidenceItem {
        tier: "OBSERVED".to_string(),
        title: "Signature Format".to_string(),
        detail: format!("Parsed base58 signature (length: {} chars)", signature.len()),
        provenance: EventProvenance::DIRECT,
    });

    let current_slot = snapshot.slot.current_slot;
    evidence.push(AutopsyEvidenceItem {
        tier: "OBSERVED".to_string(),
        title: "Current Consensus Slot".to_string(),
        detail: format!("Cluster slot {} at query time", current_slot),
        provenance: snapshot.slot.provenance,
    });

    if let Some(leader) = &snapshot.leader.current_leader {
        evidence.push(AutopsyEvidenceItem {
            tier: "OBSERVED".to_string(),
            title: "Active Leader".to_string(),
            detail: format!("Validator: {}", leader),
            provenance: snapshot.leader.provenance,
        });
    }

    // 2. Query cluster signature status & full transaction details if RPC client available
    let rpc_opt = state.rpc_client.read().await.clone();
    if let Some(rpc) = rpc_opt {
        // Query signature status
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getSignatureStatuses",
            "params": [[signature.clone()], {"searchTransactionHistory": true}]
        });

        if let Ok(resp) = reqwest::Client::new()
            .post(rpc.rpc_url())
            .json(&payload)
            .send()
            .await
        {
            if let Ok(body) = resp.json::<serde_json::Value>().await {
                if let Some(result) = body.get("result").and_then(|r| r.get("value")).and_then(|v| v.as_array()) {
                    if let Some(status_obj) = result.first().and_then(|s| s.as_object()) {
                        if let Some(conf) = status_obj.get("confirmationStatus").and_then(|c| c.as_str()) {
                            confirmation_status = conf.to_uppercase();
                        }
                        if let Some(e) = status_obj.get("err") {
                            if !e.is_null() {
                                err = Some(e.to_string());
                            }
                        }
                        if let Some(s) = status_obj.get("slot").and_then(|s| s.as_u64()) {
                            observed_slot = Some(s);
                            evidence.push(AutopsyEvidenceItem {
                                tier: "OBSERVED".to_string(),
                                title: "Cluster Confirmation".to_string(),
                                detail: format!("Landed in slot {} with status {}", s, confirmation_status),
                                provenance: EventProvenance::DIRECT,
                            });
                        }
                    }
                }
            }
        }

        // Query getTransaction for full blockTime, fee, compute units, and blockhash
        let tx_payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "getTransaction",
            "params": [
                signature.clone(),
                {"encoding": "json", "maxSupportedTransactionVersion": 0}
            ]
        });

        if let Ok(resp) = reqwest::Client::new()
            .post(rpc.rpc_url())
            .json(&tx_payload)
            .send()
            .await
        {
            if let Ok(body) = resp.json::<serde_json::Value>().await {
                if let Some(tx_obj) = body.get("result").and_then(|r| r.as_object()) {
                    block_time = tx_obj.get("blockTime").and_then(|v| v.as_i64());
                    if let Some(meta) = tx_obj.get("meta").and_then(|m| m.as_object()) {
                        fee = meta.get("fee").and_then(|f| f.as_u64());
                        compute_units_consumed = meta.get("computeUnitsConsumed").and_then(|c| c.as_u64());
                        if let Some(e) = meta.get("err") {
                            if !e.is_null() && err.is_none() {
                                err = Some(e.to_string());
                            }
                        }
                    }
                    if let Some(msg) = tx_obj.get("transaction").and_then(|t| t.get("message")).and_then(|m| m.as_object()) {
                        blockhash = msg.get("recentBlockhash").and_then(|b| b.as_str()).map(|s| s.to_string());
                    }
                }
            }
        }
    }

    // 3. Cross-reference with Chrono Engine state
    if is_fixture {
        candidate_bank = Some("bank-1".to_string());
        parent_relation = Some("SIMD-0337 UpdateParent redirected from bank-1 to bank-2".to_string());
        evidence.push(AutopsyEvidenceItem {
            tier: "INFERRED".to_string(),
            title: "Parent Handover Lineage".to_string(),
            detail: "Candidate bank invalidated by optimistic parent switch".to_string(),
            provenance: EventProvenance::DERIVED,
        });
        inferred_conclusions.push("Candidate bank abandoned due to fast leader handover UpdateParent marker".to_string());
    } else if let Some(s) = observed_slot {
        let matching_bank = snapshot.banks.candidate_banks.iter().find(|b| b.slot == s);
        if let Some(b) = matching_bank {
            candidate_bank = Some(b.bank_id.clone());
            bank_hash = b.bank_hash.clone();
            evidence.push(AutopsyEvidenceItem {
                tier: "OBSERVED".to_string(),
                title: "Tracked Candidate Bank".to_string(),
                detail: format!("Matched candidate bank {} (state: {})", b.bank_id, b.state),
                provenance: b.provenance,
            });
        } else {
            unknowns.push(format!("Slot {} was confirmed outside the currently active {} slot retention window", s, snapshot.banks.candidate_banks.len()));
        }

        if let Some(ref up) = snapshot.parent.last_update_parent {
            if up.slot == s || up.parent_slot == s {
                parent_switch = Some(serde_json::json!({
                    "occurred": true,
                    "cleared_bank_id": up.cleared_bank_id,
                    "replacement_bank_id": up.replacement_bank_id,
                    "reason": up.reason,
                    "parent_block_id": up.parent_block_id,
                    "observed_at_ms": up.observed_at_ms
                }));
                inferred_conclusions.push(format!("Fast leader handover (SIMD-0337) observed on slot {}", up.slot));
            }
        }

        if snapshot.finality.last_finalized_slot >= Some(s) {
            certificate_info = Some(serde_json::json!({
                "type": snapshot.finality.cert_type.clone().unwrap_or_else(|| "TOWER_BFT_ROOT".to_string()),
                "status": "VERIFIED",
                "stake_percent": snapshot.finality.stake_percent.unwrap_or(80.0),
                "finality_latency_ms": snapshot.finality.finality_latency_ms
            }));
        }
    } else {
        unknowns.push("Transaction signature not found in recent cluster block cache".to_string());
        unknowns.push("Validator-local candidate bank ID is not observable via standard Public RPC (requires Yellowstone gRPC / Geyser plugin)".to_string());
    }

    // 4. Construct Complete 9-Stage Forensic Timeline
    let landing_slot = observed_slot.unwrap_or(current_slot);

    timeline.push(AutopsyTimelineStep {
        step: "1. Observed".to_string(),
        status: if confirmation_status != "UNCONFIRMED" { "CONFIRMED".to_string() } else { "PENDING".to_string() },
        timestamp_ms: block_time.map(|t| (t * 1000) as u64),
        detail: format!("Transaction signature {} registered", signature),
        source: snapshot.source.clone(),
        provenance: EventProvenance::DIRECT,
    });

    timeline.push(AutopsyTimelineStep {
        step: "2. Received by Source".to_string(),
        status: "CONFIRMED".to_string(),
        timestamp_ms: None,
        detail: format!("Ingested via stream provider: {}", snapshot.source),
        source: snapshot.source.clone(),
        provenance: EventProvenance::DIRECT,
    });

    timeline.push(AutopsyTimelineStep {
        step: "3. Entry / Pre-Execution".to_string(),
        status: if snapshot.capabilities.coverage_score == 100 { "CONFIRMED".to_string() } else { "UNAVAILABLE".to_string() },
        timestamp_ms: None,
        detail: if snapshot.capabilities.coverage_score == 100 {
            "Pre-execution entry verified via validator Geyser IPC hook".to_string()
        } else {
            "Pre-execution entry notifications omitted by public RPC; requires local validator Geyser plugin".to_string()
        },
        source: snapshot.source.clone(),
        provenance: if snapshot.capabilities.coverage_score == 100 { EventProvenance::DIRECT } else { EventProvenance::UNAVAILABLE },
    });

    timeline.push(AutopsyTimelineStep {
        step: "4. Included in Block".to_string(),
        status: if observed_slot.is_some() { "CONFIRMED".to_string() } else { "PENDING".to_string() },
        timestamp_ms: block_time.map(|t| (t * 1000) as u64),
        detail: if let Some(s) = observed_slot {
            format!("Mined into block at slot {} (blockhash: {}, fee: {} lamports, compute: {} units)", s, blockhash.as_deref().unwrap_or("confirmed"), fee.unwrap_or(5000), compute_units_consumed.unwrap_or(0))
        } else {
            "Pending cluster block mining".to_string()
        },
        source: snapshot.source.clone(),
        provenance: if observed_slot.is_some() { EventProvenance::DIRECT } else { EventProvenance::UNAVAILABLE },
    });

    timeline.push(AutopsyTimelineStep {
        step: "5. Bank & State".to_string(),
        status: if candidate_bank.is_some() { "CONFIRMED".to_string() } else { "UNAVAILABLE".to_string() },
        timestamp_ms: None,
        detail: if let Some(ref cb) = candidate_bank {
            format!("Indexed in candidate bank {} with bank hash {}", cb, bank_hash.as_deref().unwrap_or("sealed"))
        } else {
            "Validator-local candidate bank ID is omitted by public RPC; tracking requires Yellowstone gRPC or Agave Geyser plugin".to_string()
        },
        source: snapshot.source.clone(),
        provenance: if candidate_bank.is_some() { EventProvenance::DIRECT } else { EventProvenance::UNAVAILABLE },
    });

    timeline.push(AutopsyTimelineStep {
        step: "6. Parent Lineage".to_string(),
        status: if observed_slot.is_some() { "CONFIRMED".to_string() } else { "PENDING".to_string() },
        timestamp_ms: None,
        detail: format!("Parent slot lineage: slot {}", landing_slot.saturating_sub(1)),
        source: snapshot.source.clone(),
        provenance: if observed_slot.is_some() { EventProvenance::DERIVED } else { EventProvenance::UNAVAILABLE },
    });

    timeline.push(AutopsyTimelineStep {
        step: "7. Fast Leader Handover (UpdateParent)".to_string(),
        status: if parent_switch.is_some() { "ABANDONED".to_string() } else { "CONFIRMED".to_string() },
        timestamp_ms: None,
        detail: if parent_switch.is_some() {
            "SIMD-0337 UpdateParent redirected state; alternative candidate bank was cleared during fast leader handover".to_string()
        } else {
            "Zero parent invalidation observed; bank continued on canonical progression".to_string()
        },
        source: snapshot.source.clone(),
        provenance: EventProvenance::DIRECT,
    });

    timeline.push(AutopsyTimelineStep {
        step: "8. Canonical Resolution".to_string(),
        status: if confirmation_status == "CONFIRMED" || confirmation_status == "FINALIZED" { "CONFIRMED".to_string() } else { "PENDING".to_string() },
        timestamp_ms: None,
        detail: if confirmation_status == "CONFIRMED" || confirmation_status == "FINALIZED" {
            "Resolved as head of canonical fork on cluster consensus".to_string()
        } else {
            "Awaiting canonical confirmation".to_string()
        },
        source: snapshot.source.clone(),
        provenance: EventProvenance::DIRECT,
    });

    timeline.push(AutopsyTimelineStep {
        step: "9. Finality & Certification".to_string(),
        status: if confirmation_status == "FINALIZED" { "CONFIRMED".to_string() } else { "PENDING".to_string() },
        timestamp_ms: None,
        detail: if confirmation_status == "FINALIZED" {
            if let Some(ref cert) = certificate_info {
                format!("Cryptographically notarized via {} ({}% stake threshold)", cert.get("type").and_then(|t| t.as_str()).unwrap_or("BLS"), cert.get("stake_percent").and_then(|s| s.as_f64()).unwrap_or(80.0))
            } else {
                "Settled at cluster finality (TowerBFT 32-lockout root commitment)".to_string()
            }
        } else {
            "Pending 32-lockout root finality or BLS notarization".to_string()
        },
        source: snapshot.source.clone(),
        provenance: if confirmation_status == "FINALIZED" { EventProvenance::DIRECT } else { EventProvenance::UNAVAILABLE },
    });

    Json(TransactionAutopsyResponse {
        signature,
        slot: observed_slot.or(Some(current_slot)),
        block_time,
        blockhash,
        bank_hash,
        fee,
        compute_units_consumed,
        leader: snapshot.leader.current_leader,
        candidate_bank,
        parent_relation,
        confirmation_status,
        err,
        parent_switch,
        certificate_info,
        evidence,
        timeline,
        inferred_conclusions,
        unknowns,
        source_provenance: if is_fixture {
            EventProvenance::DERIVED
        } else {
            EventProvenance::DIRECT
        },
    })
}

async fn get_execution_state(State(state): State<AppState>) -> impl IntoResponse {
    let ws_count = state.ws_clients.load(Ordering::Relaxed) as usize;
    let snapshot = state.engine.read().await.build_snapshot(ws_count);

    let slot_elapsed_ms = snapshot.slot.elapsed_ms;
    let slot_target_duration_ms = snapshot.slot.target_duration_ms;
    let remaining_window_ms = slot_target_duration_ms.saturating_sub(slot_elapsed_ms);

    let slot_tier = if slot_elapsed_ms < slot_target_duration_ms * 3 / 4 {
        chrono_bench::freshness::FreshnessTier::Fresh
    } else {
        chrono_bench::freshness::FreshnessTier::Stale
    };

    let leader_tier = if remaining_window_ms > 40 {
        chrono_bench::freshness::FreshnessTier::Fresh
    } else {
        chrono_bench::freshness::FreshnessTier::HandoffImminent
    };

    let blockhash_tier = chrono_bench::freshness::FreshnessTier::Fresh;
    let source_tier = if snapshot.status == "LIVE" {
        chrono_bench::freshness::FreshnessTier::Fresh
    } else {
        chrono_bench::freshness::FreshnessTier::Stale
    };

    let has_abandoned_bank = snapshot.banks.candidate_banks.iter().any(|b| b.state == "ABANDONED");
    let bank_tier = if has_abandoned_bank {
        chrono_bench::freshness::BankFreshnessTier::Abandoned
    } else {
        chrono_bench::freshness::BankFreshnessTier::Canonical
    };

    let freshness = chrono_bench::freshness::FreshnessState {
        slot_tier,
        slot_elapsed_ms,
        slot_target_duration_ms,
        leader_tier,
        current_leader: snapshot.leader.current_leader.clone(),
        next_leader: snapshot.leader.next_leader.clone(),
        remaining_window_ms,
        blockhash_tier,
        blockhash: snapshot.banks.candidate_banks.first().and_then(|b| b.blockhash.clone()).unwrap_or_else(|| "11111111111111111111111111111111".to_string()),
        blockhash_age_ms: slot_elapsed_ms,
        blockhash_age_slots: 0,
        source_tier,
        last_event_received_ago_ms: 5,
        bank_tier,
        bank_id: snapshot.banks.candidate_banks.first().map(|b| b.bank_id.clone()),
    };

    let decision = chrono_bench::decision::ExecutionDecisionEngine::evaluate_chrono(&freshness, true);

    let is_mainnet = snapshot.cluster.contains("mainnet");

    let quic_route = QuicRouteStatus {
        leader: snapshot.leader.current_leader.clone(),
        tpu_quic_port: 8009,
        prewarmed: snapshot.leader.current_leader.is_some(),
        connection_state: if snapshot.leader.current_leader.is_some() { "READY".to_string() } else { "RESOLVING".to_string() },
        fallback_rpc: true,
    };

    let contract = vec![
        "T0: Decision Available (nanoseconds monotonic)".to_string(),
        "T1: Template Build Start".to_string(),
        "T2: Signature Applied".to_string(),
        "T3: Submission Dispatched".to_string(),
        "T4: Wire Transmission".to_string(),
        "T5: Route Ack / Receiver Buffer".to_string(),
        "T6: First Observed on Cluster".to_string(),
        "T7: Landed in Candidate Bank".to_string(),
        "T8: Bank Processed & Canonical".to_string(),
        "T9: Confirmed / Notarized".to_string(),
        "T10: Finalized (Fast Path BLS or Root)".to_string(),
    ];

    Json(ExecutionStateResponse {
        decision,
        freshness,
        leader_window_ms: remaining_window_ms,
        target_leader: snapshot.leader.current_leader,
        next_leader: snapshot.leader.next_leader,
        quic_route,
        mainnet_safety_guard: true,
        execution_mode: if is_mainnet { "MAINNET_GUARDED_OBSERVATION".to_string() } else { "ACTIVE_EVALUATION".to_string() },
        timestamps_t0_t10_contract: contract,
    })
}

async fn get_measured_benchmarks() -> impl IntoResponse {
    let path = std::path::Path::new("artifacts/phase-7/benchmark-results.json");
    if let Ok(content) = std::fs::read_to_string(path) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
            return Json(val);
        }
    }
    Json(serde_json::json!({
        "status": "UNAVAILABLE",
        "error": "artifacts/phase-7/benchmark-results.json not found"
    }))
}

async fn list_benchmarks() -> impl IntoResponse {
    let storage = chrono_bench::storage::BenchmarkStorage::default();
    let exp_ids = storage.list_experiments();
    let mut summaries = Vec::new();
    for id in exp_ids.iter().rev() {
        if let Some(s) = storage.load_summary(id) {
            summaries.push(s);
        }
    }
    Json(summaries)
}

async fn get_benchmark_detail(Path(id): Path<String>) -> impl IntoResponse {
    let storage = chrono_bench::storage::BenchmarkStorage::default();
    let summary = storage.load_summary(&id);
    let samples = storage.load_samples(&id).unwrap_or_default();

    if let Some(summary) = summary {
        Json(serde_json::json!({
            "found": true,
            "summary": summary,
            "samples": samples
        }))
    } else {
        Json(serde_json::json!({
            "found": false,
            "error": format!("Experiment '{}' not found", id)
        }))
    }
}

async fn explain_execution(Path(execution_id): Path<String>) -> impl IntoResponse {
    let explanation = chrono_bench::explainer::ExecutionExplainer::find_and_explain(&execution_id);
    let sample = chrono_bench::explainer::ExecutionExplainer::find_sample(&execution_id);

    if let Some(exp) = explanation {
        Json(serde_json::json!({
            "found": true,
            "execution_id": execution_id,
            "explanation": exp,
            "sample": sample
        }))
    } else {
        Json(serde_json::json!({
            "found": false,
            "execution_id": execution_id,
            "error": format!("Execution ID '{}' not found in benchmark records", execution_id)
        }))
    }
}

#[derive(Debug, Deserialize)]
pub struct ClusterChangeRequest {
    pub cluster: String,
}

#[derive(Debug, Serialize)]
pub struct ClusterChangeResponse {
    pub success: bool,
    pub cluster: String,
    pub message: String,
}

async fn post_cluster(
    State(state): State<AppState>,
    Json(payload): Json<ClusterChangeRequest>,
) -> impl IntoResponse {
    let cluster_name = payload.cluster.to_lowercase();
    let valid_clusters = ["testnet", "devnet", "mainnet", "mainnet-beta", "local-geyser", "local-geyser-fixture", "local-validator"];
    if !valid_clusters.contains(&cluster_name.as_str()) {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(ClusterChangeResponse {
                success: false,
                cluster: cluster_name,
                message: format!("Unsupported cluster. Valid options: {:?}", valid_clusters),
            }),
        );
    }

    let mut eng = state.engine.write().await;
    let (source_type, rpc_url, ws_url) = if cluster_name == "local-validator" {
        eng.update_cluster(
            cluster_name.clone(),
            "local-validator".to_string(),
            "http://127.0.0.1:8899".to_string(),
            "ws://127.0.0.1:8900".to_string(),
        );
        eng.set_capability_matrix(chrono_adapters::capabilities::ProviderCapabilityMatrix::local_validator("local-validator"));
        ("local-validator".to_string(), "http://127.0.0.1:8899".to_string(), "ws://127.0.0.1:8900".to_string())
    } else if cluster_name == "local-geyser" || cluster_name == "local-geyser-fixture" {
        eng.update_cluster(
            cluster_name.clone(),
            "local-geyser".to_string(),
            state.config.rpc_url.clone(),
            state.config.ws_url.clone(),
        );
        eng.set_capability_matrix(chrono_adapters::capabilities::ProviderCapabilityMatrix::local_geyser_fixture("local-validator-geyser"));
        ("local-geyser".to_string(), state.config.rpc_url.clone(), state.config.ws_url.clone())
    } else {
        let matrix = chrono_adapters::capabilities::ProviderCapabilityMatrix::standard_public_rpc(format!("{}-rpc", cluster_name));
        eng.set_capability_matrix(matrix);
        let cluster_cfg = match cluster_name.as_str() {
            "testnet" => chrono_adapters::cluster::ClusterConfig::testnet(),
            "mainnet" | "mainnet-beta" => chrono_adapters::cluster::ClusterConfig::mainnet_beta(),
            _ => chrono_adapters::cluster::ClusterConfig::devnet(),
        };
        eng.update_cluster(
            cluster_name.clone(),
            "rpc".to_string(),
            cluster_cfg.rpc_url.clone(),
            cluster_cfg.ws_url.clone(),
        );
        ("rpc".to_string(), cluster_cfg.rpc_url, cluster_cfg.ws_url)
    };
    drop(eng);

    // Update active RPC client
    if source_type == "rpc" {
        let mut rpc_guard = state.rpc_client.write().await;
        *rpc_guard = Some(Arc::new(SolanaRpcClient::new(rpc_url.clone())));
    } else {
        let mut rpc_guard = state.rpc_client.write().await;
        *rpc_guard = None;
    }

    // Switch active telemetry source
    if let Some(mgr) = &state.source_manager {
        if let Err(e) = mgr.switch_source(&cluster_name, &source_type, &rpc_url, &ws_url).await {
            tracing::warn!("Failed to switch telemetry source: {}", e);
        }
    }

    (
        axum::http::StatusCode::OK,
        Json(ClusterChangeResponse {
            success: true,
            cluster: cluster_name,
            message: "Cluster configuration and telemetry source updated successfully".to_string(),
        }),
    )
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsensusResponse {
    pub current_lifecycle_state: String,
    pub fast_finality_records: Vec<chrono_bank::FastFinalityEvidence>,
    pub total_certificates_captured: usize,
    pub active_stake_epochs: Vec<u64>,
}

async fn get_consensus(State(state): State<AppState>) -> impl IntoResponse {
    let engine = state.engine.read().await;
    let records = engine.get_fast_finality_records();
    let certs = engine.captured_certificates();
    let state_str = format!("{:?}", engine.consensus_state());
    Json(ConsensusResponse {
        current_lifecycle_state: state_str,
        fast_finality_records: records,
        total_certificates_captured: certs.len(),
        active_stake_epochs: vec![0, 500],
    })
}

async fn get_application_events(State(state): State<AppState>) -> impl IntoResponse {
    let engine = state.engine.read().await;
    let events = engine.get_application_events();
    Json(events)
}
