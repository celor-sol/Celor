use crate::graph::{BankGraph, BankKey};
use chrono_core::types::{BankState, Blockhash, Slot};
use serde::{Deserialize, Serialize};

/// Statistical latency summary for measured transitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencySummary {
    pub sample_count: usize,
    /// Minimum measured latency in microseconds.
    pub min_us: u64,
    /// p50 (Median) measured latency in microseconds.
    pub p50_us: u64,
    /// p90 measured latency in microseconds.
    pub p90_us: u64,
    /// p95 measured latency in microseconds.
    pub p95_us: u64,
    /// p99 measured latency in microseconds.
    pub p99_us: u64,
    /// Maximum measured latency in microseconds.
    pub max_us: u64,
    /// Integrity label: strictly MEASURED
    pub classification: &'static str,
}

/// Recorded finality transition record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalityRecord {
    pub slot: Slot,
    pub bank_key: BankKey,
    pub blockhash: Option<Blockhash>,
    pub observed_nanos: u64,
    pub canonical_nanos: Option<u64>,
    pub finalized_nanos: u64,
    pub canonical_latency_us: Option<u64>,
    pub finality_latency_us: u64,
}

/// Finality tracking engine that computes empirical latency for certified finality.
///
/// ABSOLUTE INVARIANT:
/// Never synthesize finality or assume "150ms has elapsed, therefore finalized".
/// Finality MUST be backed by an explicit cryptographic certificate (e.g. BLS certificate)
/// or an explicit finalized root commitment from the cluster.
#[derive(Debug, Default)]
pub struct FinalityEngine {
    records: Vec<FinalityRecord>,
    canonical_latencies_us: Vec<u64>,
    finality_latencies_us: Vec<u64>,
}

impl FinalityEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records an authoritative finalization event for a bank node.
    pub fn record_finality(
        &mut self,
        graph: &mut BankGraph,
        bank_key: &BankKey,
        finalized_nanos: u64,
    ) -> Option<FinalityRecord> {
        let node = graph.get_bank_mut(bank_key)?;

        node.state = BankState::Finalized;
        node.finalized_at_nanos = Some(finalized_nanos);

        let observed_nanos = node.first_observed_nanos;
        let canonical_nanos = node.canonical_at_nanos;

        let finality_latency_us = finalized_nanos.saturating_sub(observed_nanos) / 1000;
        let canonical_latency_us =
            canonical_nanos.map(|c| c.saturating_sub(observed_nanos) / 1000);

        let record = FinalityRecord {
            slot: node.identity.slot,
            bank_key: bank_key.clone(),
            blockhash: node.identity.blockhash.clone(),
            observed_nanos,
            canonical_nanos,
            finalized_nanos,
            canonical_latency_us,
            finality_latency_us,
        };

        if let Some(c_us) = canonical_latency_us {
            self.canonical_latencies_us.push(c_us);
        }
        self.finality_latencies_us.push(finality_latency_us);
        self.records.push(record.clone());

        Some(record)
    }

    /// Returns the total number of finalized banks recorded.
    pub fn finalized_count(&self) -> usize {
        self.records.len()
    }

    /// Computes statistical latency distribution for finality latency.
    /// Returns None if sample count is 0.
    pub fn finality_latency_summary(&self) -> Option<LatencySummary> {
        Self::compute_summary(&self.finality_latencies_us)
    }

    /// Computes statistical latency distribution for canonical latency.
    pub fn canonical_latency_summary(&self) -> Option<LatencySummary> {
        Self::compute_summary(&self.canonical_latencies_us)
    }

    fn compute_summary(samples: &[u64]) -> Option<LatencySummary> {
        if samples.is_empty() {
            return None;
        }

        let mut sorted = samples.to_vec();
        sorted.sort_unstable();

        let n = sorted.len();
        let min_us = sorted[0];
        let max_us = sorted[n - 1];

        let p50_us = sorted[(n * 50) / 100];
        let p90_us = sorted[((n * 90) / 100).min(n - 1)];
        let p95_us = sorted[((n * 95) / 100).min(n - 1)];
        let p99_us = sorted[((n * 99) / 100).min(n - 1)];

        Some(LatencySummary {
            sample_count: n,
            min_us,
            p50_us,
            p90_us,
            p95_us,
            p99_us,
            max_us,
            classification: "MEASURED",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono_core::identity::BankIdentity;
    use chrono_core::types::{BankId, Blockhash, ProviderId};
    use crate::graph::BankNode;

    #[test]
    fn test_finality_engine_computes_accurate_measured_latencies() {
        let mut graph = BankGraph::new();
        let mut engine = FinalityEngine::new();
        let provider = ProviderId::new("testnet-rpc");

        let slot = Slot(100);
        let node = BankNode::new(
            BankIdentity::new(provider, Some(BankId(1)), slot, Some(Blockhash::new("FinalHash1"))),
            None,
            1_000_000_000, // 1.0s
        );
        let key = graph.insert_bank(node);

        // Mark canonical at 1.1s (100ms / 100,000us later)
        graph.mark_canonical(&key, 1_100_000_000);

        // Record finalized at 1.25s (250ms / 250,000us later)
        let record = engine
            .record_finality(&mut graph, &key, 1_250_000_000)
            .expect("record finality");

        assert_eq!(record.canonical_latency_us, Some(100_000));
        assert_eq!(record.finality_latency_us, 250_000);

        let summary = engine.finality_latency_summary().unwrap();
        assert_eq!(summary.sample_count, 1);
        assert_eq!(summary.min_us, 250_000);
        assert_eq!(summary.p50_us, 250_000);
        assert_eq!(summary.classification, "MEASURED");
    }
}
