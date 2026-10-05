use crate::experiment::{ExperimentConfig, ExperimentSummary, SampleRecord};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

pub struct BenchmarkStorage {
    base_dir: PathBuf,
}

impl Default for BenchmarkStorage {
    fn default() -> Self {
        Self {
            base_dir: PathBuf::from("artifacts/benchmarks"),
        }
    }
}

impl BenchmarkStorage {
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    pub fn experiment_dir(&self, exp_id: &str) -> PathBuf {
        self.base_dir.join(exp_id)
    }

    /// Persists the complete experiment package:
    /// - experiment.json
    /// - samples.jsonl
    /// - summary.json
    /// - README.md
    pub fn save_experiment(
        &self,
        config: &ExperimentConfig,
        samples: &[SampleRecord],
        summary: &ExperimentSummary,
    ) -> Result<PathBuf, std::io::Error> {
        let dir = self.experiment_dir(&config.id);
        fs::create_dir_all(&dir)?;

        // 1. experiment.json
        let config_file = dir.join("experiment.json");
        let config_json = serde_json::to_string_pretty(config)?;
        fs::write(config_file, config_json)?;

        // 2. samples.jsonl
        let samples_file = dir.join("samples.jsonl");
        let mut file = File::create(samples_file)?;
        for sample in samples {
            let line = serde_json::to_string(sample)?;
            writeln!(file, "{}", line)?;
        }

        // 3. summary.json
        let summary_file = dir.join("summary.json");
        let summary_json = serde_json::to_string_pretty(summary)?;
        fs::write(summary_file, summary_json)?;

        // 4. README.md
        let readme_file = dir.join("README.md");
        let readme_content = format!(
            r#"# CHRONO Benchmark Experiment: {exp_id}

> **Cluster**: `{cluster}`  
> **Source**: `{source}`  
> **Route**: `{route}`  
> **Total Samples**: `{total_samples}`  
> **Interleaved**: `{interleave}`  
> **Random Seed**: `{seed}`  
> **Chrono Advantage Verdict**: **{verdict}** ({confidence})

---

## 1. Comparative Summary Matrix

| Metric | CONTROL (Baseline) | CHRONO-AWARE | Delta (Chrono vs Control) |
|---|---|---|---|
| **Success / Confirmed Rate** | {ctrl_success:.1}% ({ctrl_conf}/{ctrl_total}) | {chrono_success:.1}% ({chrono_conf}/{chrono_total}) | **{delta_success:+.1}%** |
| **Confirmation Latency (p50)** | {ctrl_conf_p50:.1}ms | {chrono_conf_p50:.1}ms | **{delta_conf:+.1}ms** |
| **Confirmation Latency (p95)** | {ctrl_conf_p95:.1}ms | {chrono_conf_p95:.1}ms | — |
| **Landing Latency (p50)** | {ctrl_land_p50:.1}ms | {chrono_land_p50:.1}ms | — |
| **Slot Delta to Landing (p50)** | +{ctrl_slot_p50:.1} slots | +{chrono_slot_p50:.1} slots | — |
| **Stale Blockhash Errors** | {ctrl_bh_err} | {chrono_bh_err} | **{delta_bh:+}** |
| **Leader Handoff Misses** | {ctrl_miss} | {chrono_miss} | **{delta_miss:+}** |
| **Total Retries** | {ctrl_retries} | {chrono_retries} | — |

---

## 2. Limitations & Environment Notes
{limitations}
"#,
            exp_id = config.id,
            cluster = config.cluster,
            source = config.source,
            route = config.route,
            total_samples = samples.len(),
            interleave = config.interleave_mode,
            seed = config.random_seed,
            verdict = summary.chrono_advantage_verdict,
            confidence = summary.statistical_confidence,
            ctrl_success = summary.control_summary.success_rate,
            ctrl_conf = summary.control_summary.confirmed_count,
            ctrl_total = summary.control_summary.total_samples,
            chrono_success = summary.chrono_summary.success_rate,
            chrono_conf = summary.chrono_summary.confirmed_count,
            chrono_total = summary.chrono_summary.total_samples,
            delta_success = summary.delta_success_rate_percent,
            ctrl_conf_p50 = summary.control_summary.submit_to_confirmed_ms_p50,
            chrono_conf_p50 = summary.chrono_summary.submit_to_confirmed_ms_p50,
            delta_conf = summary.delta_confirmation_latency_ms,
            ctrl_conf_p95 = summary.control_summary.submit_to_confirmed_ms_p95,
            chrono_conf_p95 = summary.chrono_summary.submit_to_confirmed_ms_p95,
            ctrl_land_p50 = summary.control_summary.submit_to_landed_ms_p50,
            chrono_land_p50 = summary.chrono_summary.submit_to_landed_ms_p50,
            ctrl_slot_p50 = summary.control_summary.slot_delta_landed_p50,
            chrono_slot_p50 = summary.chrono_summary.slot_delta_landed_p50,
            ctrl_bh_err = summary.control_summary.stale_blockhash_errors,
            chrono_bh_err = summary.chrono_summary.stale_blockhash_errors,
            delta_bh = (summary.chrono_summary.stale_blockhash_errors as i64)
                - (summary.control_summary.stale_blockhash_errors as i64),
            ctrl_miss = summary.control_summary.leader_handoff_misses,
            chrono_miss = summary.chrono_summary.leader_handoff_misses,
            delta_miss = (summary.chrono_summary.leader_handoff_misses as i64)
                - (summary.control_summary.leader_handoff_misses as i64),
            ctrl_retries = summary.control_summary.retry_count_total,
            chrono_retries = summary.chrono_summary.retry_count_total,
            limitations = if summary.limitations.is_empty() {
                "- None noted during this experiment.".to_string()
            } else {
                summary
                    .limitations
                    .iter()
                    .map(|l| format!("- {}", l))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        );

        fs::write(readme_file, readme_content)?;
        Ok(dir)
    }

    /// Lists all stored experiment IDs in chronological order.
    pub fn list_experiments(&self) -> Vec<String> {
        let mut list = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.base_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        list.push(name.to_string());
                    }
                }
            }
        }
        list.sort();
        list
    }

    pub fn load_summary(&self, exp_id: &str) -> Option<ExperimentSummary> {
        let file = self.experiment_dir(exp_id).join("summary.json");
        let content = fs::read_to_string(file).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn load_samples(&self, exp_id: &str) -> Option<Vec<SampleRecord>> {
        let file = self.experiment_dir(exp_id).join("samples.jsonl");
        let f = File::open(file).ok()?;
        let reader = BufReader::new(f);
        let mut samples = Vec::new();
        for line in reader.lines().map_while(Result::ok) {
            if let Ok(s) = serde_json::from_str::<SampleRecord>(&line) {
                samples.push(s);
            }
        }
        Some(samples)
    }
}
