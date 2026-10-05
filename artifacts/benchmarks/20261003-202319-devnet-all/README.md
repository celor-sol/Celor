# CHRONO Benchmark Experiment: 20261003-202319-devnet-all

> **Cluster**: `devnet`  
> **Source**: `rpc-stream`  
> **Route**: `phase5-devnet-all`  
> **Total Samples**: `200`  
> **Interleaved**: `true`  
> **Random Seed**: `42`  
> **Chrono Advantage Verdict**: **CONDITIONAL** (Moderate (Advantage observed under specific timing conditions))

---

## 1. Comparative Summary Matrix

| Metric | CONTROL (Baseline) | CHRONO-AWARE | Delta (Chrono vs Control) |
|---|---|---|---|
| **Success / Confirmed Rate** | 41.0% (41/100) | 40.0% (40/100) | **-1.0%** |
| **Confirmation Latency (p50)** | 2101.7ms | 1209.1ms | **-892.6ms** |
| **Confirmation Latency (p95)** | 7735.1ms | 10268.4ms | — |
| **Landing Latency (p50)** | 1366.1ms | 785.9ms | — |
| **Slot Delta to Landing (p50)** | +5.0 slots | +6.0 slots | — |
| **Stale Blockhash Errors** | 0 | 0 | **+0** |
| **Leader Handoff Misses** | 0 | 0 | **+0** |
| **Total Retries** | 0 | 0 | — |

---

## 2. Limitations & Environment Notes
- Public RPC endpoint rate-limits bound transaction burst velocity
