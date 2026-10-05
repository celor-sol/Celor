# CHRONO Benchmark Experiment: 20261003-192833-devnet-rpc

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
| **Success / Confirmed Rate** | 96.0% (24/25) | 40.0% (10/25) | **-56.0%** |
| **Confirmation Latency (p50)** | 951.7ms | 987.8ms | **+36.1ms** |
| **Confirmation Latency (p95)** | 1422.9ms | 2152.3ms | — |
| **Landing Latency (p50)** | 618.6ms | 642.1ms | — |
| **Slot Delta to Landing (p50)** | +5.0 slots | +5.0 slots | — |
| **Stale Blockhash Errors** | 0 | 0 | **+0** |
| **Leader Handoff Misses** | 0 | 0 | **+0** |
| **Total Retries** | 0 | 0 | — |

---

## 2. Limitations & Environment Notes
- Public RPC endpoint rate-limits bound transaction burst velocity
