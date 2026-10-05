# CHRONO QUIC Route Specification & Operational Guide

## 1. Overview

The `DirectLeaderQuicRoute` provides low-latency, zero-hop transaction delivery directly to the active Solana validator's TPU QUIC port, bypassing public JSON-RPC relay queues.

---

## 2. CLI Diagnostics & Operations

### 2.1 Route Status Diagnostic
To inspect cluster node topology, resolve the current leader's TPU QUIC socket, test prewarming, and measure RPC RTT:

```bash
chrono route status --cluster devnet
```

**Example Output**:
```text
=== CHRONO EXECUTION ROUTE DIAGNOSTIC ===
Cluster:          devnet
RPC Endpoint:     https://api.devnet.solana.com
RPC Ping Latency: 692 ms

Resolving cluster leader and TPU topology...
✓ Cluster Nodes:  155 active validators cached
✓ Current Slot:   332194812
✓ Current Leader: Gt8...xK9
✓ TPU QUIC Skt:   38.246.35.41:8003 (Active)
✓ Prewarm Result: Connection prewarmed in 0.91ms
✓ Fallback Mode:  QuicThenRpc
✓ Targeting Mode: CurrentPlusNext
=========================================
```

### 2.2 Benchmarking With Specific Routes
To execute controlled execution benchmarks across specific transport layers:

```bash
# 1. Full 4-way 2x2 Factorial Matrix (Interleaved Control RPC, Chrono RPC, Control QUIC, Chrono QUIC)
chrono bench run --cluster devnet --samples 200 --route all --max-tps 1

# 2. QUIC-only A/B comparison (Control QUIC vs Chrono QUIC)
chrono bench run --cluster devnet --samples 100 --route quic --max-tps 1

# 3. RPC-only legacy baseline (Control RPC vs Chrono RPC)
chrono bench run --cluster devnet --samples 100 --route rpc --max-tps 1
```

---

## 3. Configuration & Policies

### 3.1 Fallback Policy (`FallbackPolicy`)
Controls behavior when the leader's TPU socket cannot be reached or drops packets:

- **`QuicOnly`**:
  Strict zero-fallback mode. If QUIC connection establishment or send fails, the sample is recorded as an execution error. Used for strict protocol latency benchmarking.
- **`QuicThenRpc`** (Default):
  Production resilient mode. If direct QUIC transmission times out or fails (e.g., node firewall blocks UDP 8003 or validator drops connection), the route automatically falls back to JSON-RPC `sendTransaction`. Every fallback event increments `fallback_count` and is logged in telemetry.
- **`RpcOnly`**:
  Bypasses QUIC entirely, submitting transactions via standard HTTP JSON-RPC POST.

### 3.2 Leader Targeting Policy (`TargetingPolicy`)
Controls dispatch timing relative to the 400ms Solana slot boundary:

- **`CurrentLeaderOnly`**:
  Transmits the transaction solely to the leader of the current slot.
- **`CurrentPlusNext`**:
  When Chrono indicates that fewer than 80ms (or 20% of slot time) remain in the active slot, the route dispatches the transaction to both the current leader and the upcoming slot leader. This mitigates "leader boundary drop," where a leader stops processing new transactions during the final banking stage window.

---

## 4. Connection Cache & Warmup Strategy

QUIC requires an initial TLS 1.3 cryptographic handshake (1 RTT), incurring 300ms–700ms cold latency depending on network distance to the validator.

### 4.1 Chrono Lookahead Prewarming
Because Chrono tracks the Solana leader schedule 4 slots in advance, the `DirectLeaderQuicRoute` proactively prewarms connections to upcoming leaders:
1. Leader schedule lookahead identifies the validator for slot $S+4$.
2. The route initiates a non-blocking QUIC connection prewarm in a background worker thread.
3. When the leader handover occurs at slot $S+4$, the connection is already in the `QuicConnectionCache`.
4. Subsequent transaction dispatch executes with **zero handshake delay** (< 1ms local enqueue).
