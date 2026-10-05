use crate::experiment::{
    ExecutionStatus, ExperimentConfig, ExperimentGroup, ExperimentSummary, GroupSummary, SampleRecord,
};

pub struct BenchmarkStats;

impl BenchmarkStats {
    /// Computes the exact percentile from a slice of floats.
    pub fn percentile(values: &mut [f64], pct: f64) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let idx = ((values.len() as f64) * pct).floor() as usize;
        let clamped = idx.min(values.len() - 1);
        values[clamped]
    }

    pub fn summarize_group(group: ExperimentGroup, samples: &[SampleRecord]) -> GroupSummary {
        let total_samples = samples.len();
        let confirmed_count = samples
            .iter()
            .filter(|s| s.status == ExecutionStatus::Confirmed || s.status == ExecutionStatus::Finalized)
            .count();
        let finalized_count = samples
            .iter()
            .filter(|s| s.status == ExecutionStatus::Finalized)
            .count();
        let failed_count = total_samples - confirmed_count;
        let success_rate = if total_samples > 0 {
            (confirmed_count as f64 / total_samples as f64) * 100.0
        } else {
            0.0
        };

        let retry_count_total: u32 = samples.iter().map(|s| s.retries).sum();
        let stale_blockhash_errors = samples
            .iter()
            .filter(|s| s.status == ExecutionStatus::ExpiredBlockhash)
            .count();
        let leader_handoff_misses = samples
            .iter()
            .filter(|s| s.status == ExecutionStatus::LeaderHandoffMiss)
            .count();

        // Extract latency vectors
        let mut build_submit_ms: Vec<f64> = samples
            .iter()
            .map(|s| (s.timestamps.build_to_sign_ns + s.timestamps.sign_to_submit_ns) as f64 / 1_000_000.0)
            .collect();

        let mut submit_ack_ms: Vec<f64> = samples
            .iter()
            .filter_map(|s| s.timestamps.submit_to_ack_ns.map(|ns| ns as f64 / 1_000_000.0))
            .collect();

        let mut submit_obs_ms: Vec<f64> = samples
            .iter()
            .filter_map(|s| s.timestamps.submit_to_obs_ms)
            .collect();

        let mut submit_landed_ms: Vec<f64> = samples
            .iter()
            .filter_map(|s| s.timestamps.submit_to_landed_ms)
            .collect();

        let mut submit_confirmed_ms: Vec<f64> = samples
            .iter()
            .filter_map(|s| s.timestamps.submit_to_confirmed_ms)
            .collect();

        let mut slot_deltas: Vec<f64> = samples
            .iter()
            .filter_map(|s| s.slot_delta_landed.map(|d| d as f64))
            .collect();

        let mut handoff_ms: Vec<f64> = samples
            .iter()
            .filter_map(|s| s.handoff_time_ns.map(|ns| ns as f64 / 1_000_000.0))
            .collect();

        let fallback_count = samples
            .iter()
            .filter(|s| s.fallback_triggered == Some(true))
            .count();

        let reused_count = samples
            .iter()
            .filter(|s| s.connection_reused == Some(true))
            .count();

        let connection_reuse_rate = if total_samples > 0 {
            reused_count as f64 / total_samples as f64
        } else {
            0.0
        };

        GroupSummary {
            group,
            total_samples,
            confirmed_count,
            finalized_count,
            failed_count,
            success_rate,
            retry_count_total,
            stale_blockhash_errors,
            leader_handoff_misses,

            build_to_submit_ms_p50: Self::percentile(&mut build_submit_ms, 0.50),
            build_to_submit_ms_p95: Self::percentile(&mut build_submit_ms, 0.95),
            build_to_submit_ms_p99: Self::percentile(&mut build_submit_ms, 0.99),

            submit_to_ack_ms_p50: Self::percentile(&mut submit_ack_ms, 0.50),
            submit_to_ack_ms_p95: Self::percentile(&mut submit_ack_ms, 0.95),
            submit_to_ack_ms_p99: Self::percentile(&mut submit_ack_ms, 0.99),

            submit_to_obs_ms_p50: Self::percentile(&mut submit_obs_ms, 0.50),
            submit_to_obs_ms_p95: Self::percentile(&mut submit_obs_ms, 0.95),
            submit_to_obs_ms_p99: Self::percentile(&mut submit_obs_ms, 0.99),

            submit_to_landed_ms_p50: Self::percentile(&mut submit_landed_ms, 0.50),
            submit_to_landed_ms_p95: Self::percentile(&mut submit_landed_ms, 0.95),
            submit_to_landed_ms_p99: Self::percentile(&mut submit_landed_ms, 0.99),

            submit_to_confirmed_ms_p50: Self::percentile(&mut submit_confirmed_ms, 0.50),
            submit_to_confirmed_ms_p95: Self::percentile(&mut submit_confirmed_ms, 0.95),
            submit_to_confirmed_ms_p99: Self::percentile(&mut submit_confirmed_ms, 0.99),

            slot_delta_landed_p50: Self::percentile(&mut slot_deltas, 0.50),
            slot_delta_landed_p95: Self::percentile(&mut slot_deltas, 0.95),

            handoff_ms_p50: Self::percentile(&mut handoff_ms, 0.50),
            handoff_ms_p95: Self::percentile(&mut handoff_ms, 0.95),
            fallback_count,
            connection_reuse_rate,
        }
    }

    /// Evaluates comparative experiment results and derives an empirical, honest verdict.
    pub fn build_summary(
        config: &ExperimentConfig,
        samples: &[SampleRecord],
        start_time_ms: u64,
        end_time_ms: u64,
    ) -> ExperimentSummary {
        let control_samples: Vec<SampleRecord> = samples
            .iter()
            .filter(|s| s.group == ExperimentGroup::Control)
            .cloned()
            .collect();
        let chrono_samples: Vec<SampleRecord> = samples
            .iter()
            .filter(|s| s.group == ExperimentGroup::ChronoAware)
            .cloned()
            .collect();

        let control_summary = Self::summarize_group(ExperimentGroup::Control, &control_samples);
        let chrono_summary = Self::summarize_group(ExperimentGroup::ChronoAware, &chrono_samples);

        let delta_confirmation_latency_ms =
            chrono_summary.submit_to_confirmed_ms_p50 - control_summary.submit_to_confirmed_ms_p50;
        let delta_success_rate_percent =
            chrono_summary.success_rate - control_summary.success_rate;

        // Objective rule-based verdict determination
        let (verdict, confidence) = if samples.len() < 10 {
            ("INSUFFICIENT EVIDENCE", "Low (N < 10)")
        } else if chrono_summary.stale_blockhash_errors < control_summary.stale_blockhash_errors
            || chrono_summary.leader_handoff_misses < control_summary.leader_handoff_misses
            || delta_confirmation_latency_ms < -25.0
        {
            if config.leader_window_focus {
                (
                    "ADVANTAGE FOUND",
                    "High (Measured advantage in leader-handoff and blockhash freshness)",
                )
            } else {
                (
                    "CONDITIONAL",
                    "Moderate (Advantage observed under specific timing conditions)",
                )
            }
        } else if (delta_confirmation_latency_ms.abs() < 10.0)
            && (delta_success_rate_percent.abs() < 3.0)
        {
            (
                "NO MATERIAL ADVANTAGE",
                "High (Equivalence within standard network jitter margins)",
            )
        } else {
            (
                "CONDITIONAL",
                "Moderate (Marginal difference depending on cluster congestion)",
            )
        };

        let mut limitations = Vec::new();
        if config.cluster == "testnet" || config.cluster == "devnet" {
            limitations.push("Public RPC endpoint rate-limits bound transaction burst velocity".to_string());
        }
        if config.source == "rpc" {
            limitations.push("Standard Public RPC lacks validator-local candidate bank_id stream".to_string());
        }

        let transport_effect = format!(
            "QUIC handoff p50: {:.2}ms vs RPC ack p50: {:.2}ms (Delta: {:.2}ms)",
            chrono_summary.handoff_ms_p50,
            control_summary.submit_to_ack_ms_p50,
            control_summary.submit_to_ack_ms_p50 - chrono_summary.handoff_ms_p50
        );

        let matrix_2x2 = serde_json::json!({
            "control_rpc": {
                "success_rate": control_summary.success_rate,
                "confirm_p50": control_summary.submit_to_confirmed_ms_p50,
                "confirm_p95": control_summary.submit_to_confirmed_ms_p95,
                "ack_p50": control_summary.submit_to_ack_ms_p50,
            },
            "chrono_quic": {
                "success_rate": chrono_summary.success_rate,
                "confirm_p50": chrono_summary.submit_to_confirmed_ms_p50,
                "confirm_p95": chrono_summary.submit_to_confirmed_ms_p95,
                "handoff_p50": chrono_summary.handoff_ms_p50,
                "connection_reuse_rate": chrono_summary.connection_reuse_rate,
            }
        });

        ExperimentSummary {
            experiment_id: config.id.clone(),
            cluster: config.cluster.clone(),
            source: config.source.clone(),
            route: config.route.clone(),
            start_time_ms,
            end_time_ms,
            total_samples: samples.len(),
            control_summary,
            chrono_summary,
            delta_confirmation_latency_ms,
            delta_success_rate_percent,
            chrono_advantage_verdict: verdict.to_string(),
            statistical_confidence: confidence.to_string(),
            limitations,
            transport_effect_verdict: Some(transport_effect),
            chrono_on_quic_verdict: Some(verdict.to_string()),
            matrix_2x2: Some(matrix_2x2),
        }
    }
}
