use chrono_core::types::FieldProvenance;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExperimentGroup {
    Control,
    ChronoAware,
}

impl std::fmt::Display for ExperimentGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExperimentGroup::Control => write!(f, "CONTROL"),
            ExperimentGroup::ChronoAware => write!(f, "CHRONO_AWARE"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Landed,
    Confirmed,
    Finalized,
    ExpiredBlockhash,
    LeaderHandoffMiss,
    Timeout,
    Rejected,
    Dropped,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    pub id: String,
    pub description: String,
    pub cluster: String,
    pub source: String,
    pub route: String,
    pub transaction_type: String,
    pub sample_count: usize,
    pub max_tps: f64,
    pub timeout_ms: u64,
    pub random_seed: u64,
    pub interleave_mode: bool,
    pub leader_window_focus: bool,
    pub bank_awareness: bool,
    pub finality_target: String,
}

impl Default for ExperimentConfig {
    fn default() -> Self {
        Self {
            id: format!("exp-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S")),
            description: "Default Interleaved Control vs Chrono-Aware Benchmark".to_string(),
            cluster: "testnet".to_string(),
            source: "rpc".to_string(),
            route: "rpc".to_string(),
            transaction_type: "sol_transfer".to_string(),
            sample_count: 50,
            max_tps: 5.0,
            timeout_ms: 15_000,
            random_seed: 42,
            interleave_mode: true,
            leader_window_focus: false,
            bank_awareness: false,
            finality_target: "confirmed".to_string(),
        }
    }
}

/// Exact high-resolution latency timestamps across all 11 lifecycle boundaries (T0 through T10).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyBreakdown {
    pub t0_decision_available_ns: u64,
    pub t1_template_build_start_ns: u64,
    pub t2_signed_ns: u64,
    pub t3_submission_start_ns: u64,
    pub t4_submission_sent_ns: u64,
    pub t5_route_ack_ns: Option<u64>,
    pub t6_first_observed_ns: Option<u64>,
    pub t7_landed_slot_ns: Option<u64>,
    pub t8_processed_ns: Option<u64>,
    pub t9_confirmed_ns: Option<u64>,
    pub t10_finalized_ns: Option<u64>,

    // Computed stage durations
    pub decision_to_build_ns: u64,
    pub build_to_sign_ns: u64,
    pub sign_to_submit_ns: u64,
    pub submit_to_ack_ns: Option<u64>,
    pub submit_to_obs_ms: Option<f64>,
    pub submit_to_landed_ms: Option<f64>,
    pub submit_to_confirmed_ms: Option<f64>,
    pub submit_to_finalized_ms: Option<f64>,
}

/// Complete raw telemetry record for an individual benchmark sample.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleRecord {
    pub sample_id: String,
    pub execution_id: String,
    pub experiment_id: String,
    pub sample_index: usize,
    pub group: ExperimentGroup,
    pub timestamps: LatencyBreakdown,
    pub slot_at_submission: u64,
    pub slot_at_landing: Option<u64>,
    pub slot_at_confirmation: Option<u64>,
    pub slot_delta_landed: Option<i64>,
    pub slot_delta_confirmed: Option<i64>,
    pub leader_at_submission: Option<String>,
    pub leader_at_landing: Option<String>,
    pub landed_on_intended_leader: Option<bool>,
    pub remaining_slot_time_ms: Option<u64>,
    pub blockhash_age_ms: u64,
    pub candidate_bank_id: Option<String>,
    pub update_parent_observed: bool,
    pub retries: u32,
    pub status: ExecutionStatus,
    pub provenance: FieldProvenance,
    pub decision_reason: String,
    pub comparative_explanation: String,
    pub signature: Option<String>,
    pub error_reason: Option<String>,
    #[serde(default)]
    pub route_name: Option<String>,
    #[serde(default)]
    pub transport_type: Option<String>,
    #[serde(default)]
    pub handshake_time_ns: Option<u64>,
    #[serde(default)]
    pub handoff_time_ns: Option<u64>,
    #[serde(default)]
    pub target_leader: Option<String>,
    #[serde(default)]
    pub tpu_socket: Option<String>,
    #[serde(default)]
    pub connection_reused: Option<bool>,
    #[serde(default)]
    pub fallback_triggered: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSummary {
    pub group: ExperimentGroup,
    pub total_samples: usize,
    pub confirmed_count: usize,
    pub finalized_count: usize,
    pub failed_count: usize,
    pub success_rate: f64,
    pub retry_count_total: u32,
    pub stale_blockhash_errors: usize,
    pub leader_handoff_misses: usize,

    // Latency Distributions (p50, p90, p95, p99, mean, min, max) in ms
    pub build_to_submit_ms_p50: f64,
    pub build_to_submit_ms_p95: f64,
    pub build_to_submit_ms_p99: f64,

    pub submit_to_ack_ms_p50: f64,
    pub submit_to_ack_ms_p95: f64,
    pub submit_to_ack_ms_p99: f64,

    pub submit_to_obs_ms_p50: f64,
    pub submit_to_obs_ms_p95: f64,
    pub submit_to_obs_ms_p99: f64,

    pub submit_to_landed_ms_p50: f64,
    pub submit_to_landed_ms_p95: f64,
    pub submit_to_landed_ms_p99: f64,

    pub submit_to_confirmed_ms_p50: f64,
    pub submit_to_confirmed_ms_p95: f64,
    pub submit_to_confirmed_ms_p99: f64,

    // Slot Deltas
    pub slot_delta_landed_p50: f64,
    pub slot_delta_landed_p95: f64,

    // Phase 5 QUIC and Transport Metrics
    #[serde(default)]
    pub handoff_ms_p50: f64,
    #[serde(default)]
    pub handoff_ms_p95: f64,
    #[serde(default)]
    pub fallback_count: usize,
    #[serde(default)]
    pub connection_reuse_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentSummary {
    pub experiment_id: String,
    pub cluster: String,
    pub source: String,
    pub route: String,
    pub start_time_ms: u64,
    pub end_time_ms: u64,
    pub total_samples: usize,
    pub control_summary: GroupSummary,
    pub chrono_summary: GroupSummary,
    pub delta_confirmation_latency_ms: f64,
    pub delta_success_rate_percent: f64,
    pub chrono_advantage_verdict: String, // "ADVANTAGE FOUND", "NO MATERIAL ADVANTAGE", "CONDITIONAL", "INSUFFICIENT EVIDENCE"
    pub statistical_confidence: String,
    pub limitations: Vec<String>,
    #[serde(default)]
    pub transport_effect_verdict: Option<String>,
    #[serde(default)]
    pub chrono_on_quic_verdict: Option<String>,
    #[serde(default)]
    pub matrix_2x2: Option<serde_json::Value>,
}
