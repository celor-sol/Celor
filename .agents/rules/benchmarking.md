---
trigger: always_on
description: Mandatory benchmarking standards, latency classifications, and statistical rules for CHRONO
---

# CHRONO Benchmarking & Latency Rules

## 1. Metric Classification Standard
Every latency or timing assertion MUST be tagged with one of the following:
- `[MEASURED]`: Empirically recorded from monotonic hardware clocks during real execution.
- `[ESTIMATED]`: Analytically calculated from verified models and known physical constants.
- `[SIMULATED]`: Generated in synthetic tests or mock environments.
- `[UNKNOWN]`: Unverified or unmeasured.

## 2. Absolute Prohibition on Simulated Speed Claims
- NEVER cite or present simulated benchmarks as evidence of real-world speed or competitive performance.
- Simulated tests are strictly limited to logical regression testing and invariant checking.

## 3. Statistical Rigor Requirements
- Any benchmark report claiming performance optimization must include:
  - Minimum sample size: **N ≥ 10,000 slots or events**.
  - Percentiles: **Min, p50 (Median), p90, p95, p99, Max**.
  - Event failure / drop / timeout rate.
- Micro-benchmarks under artificial single-run loops are invalid for performance claims.

## 4. Empirical A/B Testing
- To claim that an Alpenglow-aware algorithm or CHRONO module outperforms legacy baselines, both pathways must be executed concurrently on the exact same stream feed under identical hardware and network conditions.
- Keep legacy control paths operational to support ongoing comparative profiling.
