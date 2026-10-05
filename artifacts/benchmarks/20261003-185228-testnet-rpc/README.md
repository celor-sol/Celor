# CHRONO Benchmark Experiment: 20261003-185228-testnet-rpc

> **Status**: `INVALID_UNFUNDED_RUN`  
> **Exclusion Notice**: **EXCLUDED FROM EXECUTION PERFORMANCE CLAIMS**. Both Control and Chrono had 0% execution success because the benchmark wallet had 0 SOL for transaction fees. No transactions executed on-chain.  
> **Cluster**: `testnet`  
> **Source**: `rpc-stream`  
> **Route**: `rpc-standard-vs-chrono`  
> **Total Samples**: `20`  
> **Interleaved**: `true`  
> **Random Seed**: `42`  
> **Chrono Advantage Verdict**: **INVALID_UNFUNDED_RUN (EXCLUDED)**

---

## 1. Comparative Summary Matrix

| Metric | CONTROL (Baseline) | CHRONO-AWARE | Delta (Chrono vs Control) |
|---|---|---|---|
| **Success / Confirmed Rate** | 0.0% (0/10) | 0.0% (0/10) | **+0.0%** |
| **Confirmation Latency (p50)** | 0.0ms | 0.0ms | **+0.0ms** |
| **Confirmation Latency (p95)** | 0.0ms | 0.0ms | — |
| **Landing Latency (p50)** | 0.0ms | 0.0ms | — |
| **Slot Delta to Landing (p50)** | +0.0 slots | +0.0 slots | — |
| **Stale Blockhash Errors** | 0 | 0 | **+0** |
| **Leader Handoff Misses** | 0 | 0 | **+0** |
| **Total Retries** | 0 | 0 | — |

---

## 2. Limitations & Environment Notes
- Public RPC endpoint rate-limits bound transaction burst velocity
