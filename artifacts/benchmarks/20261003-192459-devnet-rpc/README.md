# CHRONO Benchmark Experiment: 20261003-192459-devnet-rpc

> **Cluster**: `devnet`  
> **Source**: `rpc-stream`  
> **Route**: `rpc-standard-vs-chrono`  
> **Total Samples**: `50`  
> **Interleaved**: `true`  
> **Random Seed**: `42`  
> **Chrono Advantage Verdict**: **CONDITIONAL** (Moderate (Marginal difference depending on cluster congestion))

---

## 1. Comparative Summary Matrix

| Metric | CONTROL (Baseline) | CHRONO-AWARE | Delta (Chrono vs Control) |
|---|---|---|---|
| **Success / Confirmed Rate** | 88.0% (22/25) | 60.0% (15/25) | **-28.0%** |
| **Confirmation Latency (p50)** | 1062.8ms | 1125.6ms | **+62.9ms** |
| **Confirmation Latency (p95)** | 7425.2ms | 9869.5ms | — |
| **Landing Latency (p50)** | 690.8ms | 731.7ms | — |
| **Slot Delta to Landing (p50)** | +5.0 slots | +5.0 slots | — |
| **Stale Blockhash Errors** | 0 | 0 | **+0** |
| **Leader Handoff Misses** | 0 | 0 | **+0** |
| **Total Retries** | 0 | 0 | — |

---

## 2. Limitations & Environment Notes
- Public RPC endpoint rate-limits bound transaction burst velocity
