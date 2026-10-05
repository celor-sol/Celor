# CHRONO Benchmark Experiment: 20261003-202217-devnet-all

> **Cluster**: `devnet`  
> **Source**: `rpc-stream`  
> **Route**: `phase5-devnet-all`  
> **Total Samples**: `8`  
> **Interleaved**: `true`  
> **Random Seed**: `42`  
> **Chrono Advantage Verdict**: **INSUFFICIENT EVIDENCE** (Low (N < 10))

---

## 1. Comparative Summary Matrix

| Metric | CONTROL (Baseline) | CHRONO-AWARE | Delta (Chrono vs Control) |
|---|---|---|---|
| **Success / Confirmed Rate** | 75.0% (3/4) | 75.0% (3/4) | **+0.0%** |
| **Confirmation Latency (p50)** | 3657.8ms | 941.5ms | **-2716.3ms** |
| **Confirmation Latency (p95)** | 5302.0ms | 5790.9ms | — |
| **Landing Latency (p50)** | 2377.6ms | 612.0ms | — |
| **Slot Delta to Landing (p50)** | +5.0 slots | +5.0 slots | — |
| **Stale Blockhash Errors** | 0 | 0 | **+0** |
| **Leader Handoff Misses** | 0 | 0 | **+0** |
| **Total Retries** | 0 | 0 | — |

---

## 2. Limitations & Environment Notes
- Public RPC endpoint rate-limits bound transaction burst velocity
