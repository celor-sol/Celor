# CHRONO Benchmark Experiment: test-exp-60643

> **Cluster**: `testnet`  
> **Source**: `rpc`  
> **Route**: `rpc`  
> **Total Samples**: `10`  
> **Interleaved**: `true`  
> **Random Seed**: `42`  
> **Chrono Advantage Verdict**: **CONDITIONAL** (Moderate (Advantage observed under specific timing conditions))

---

## 1. Comparative Summary Matrix

| Metric | CONTROL (Baseline) | CHRONO-AWARE | Delta (Chrono vs Control) |
|---|---|---|---|
| **Success / Confirmed Rate** | 100.0% (5/5) | 100.0% (5/5) | **+0.0%** |
| **Confirmation Latency (p50)** | 310.0ms | 260.0ms | **+50.0ms** |
| **Confirmation Latency (p95)** | 310.0ms | 260.0ms | — |
| **Landing Latency (p50)** | 190.0ms | 145.0ms | — |
| **Slot Delta to Landing (p50)** | +2.0 slots | +1.0 slots | — |
| **Stale Blockhash Errors** | 0 | 0 | **+0** |
| **Leader Handoff Misses** | 0 | 0 | **+0** |
| **Total Retries** | 0 | 0 | — |

---

## 2. Limitations & Environment Notes
- Public RPC endpoint rate-limits bound transaction burst velocity
- Standard Public RPC lacks validator-local candidate bank_id stream
