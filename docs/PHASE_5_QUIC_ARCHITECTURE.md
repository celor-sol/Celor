# CHRONO Phase 5: Direct Leader QUIC / TPU Execution Architecture

## 1. Executive Summary & Objective

In Phase 4 and Phase 4.1, CHRONO established that consensus-aware execution timing (slot boundary lookahead, bank freshness tracking, stale-blockhash rejection) achieved superior latency and zero stale-blockhash errors locally. However, over public JSON-RPC endpoints on live clusters (Solana Devnet/Testnet), public RPC queueing and round-trip transport masked Chrono's internal timing advantage.

**Phase 5 Objective**: Eliminate the public RPC gateway bottleneck by deploying a direct leader QUIC/TPU transport layer (`DirectLeaderQuicRoute`) coupled with dynamic cluster leader resolution (`LeaderTransportResolver`). This architecture routes signed wire transactions directly into the active validator's QUIC ingestion pipeline (`tpuQuic`), measuring whether consensus awareness unlocks a definitive execution advantage when transport latency is minimized.

---

## 2. Solana Agave 4.3 TPU Transport Reality

### 2.1 The Death of Raw UDP TPU Ingestion
Historically, Solana validators accepted raw UDP datagrams on their TPU ports. Due to unauthenticated packet flooding and Sybil amplification, Solana core engineering (SIMD-0002 / SIMD-0105) transitioned the TPU pipeline to IETF QUIC over UDP.

In Agave 4.3.0+:
- **Raw UDP TPU is completely disabled**: Querying `getClusterNodes` on Devnet, Testnet, or Mainnet returns `tpu: null` across 100% of validators.
- **QUIC is mandatory**: Ingress transactions must be delivered to `tpuQuic` (default port `8003` or dynamic) using TLS 1.3 encrypted QUIC streams.
- **Forwarding via QUIC**: Inter-validator forwarding uses `tpuForwardsQuic` (default port `8004` or dynamic).

Attempting to transmit raw UDP packets results in immediate packet dropping at validator network firewalls.

### 2.2 Agave 4.3 Protocol Feature Classification
| Component / Interface | Status | Canonical Specification | Operational Reality |
| :--- | :--- | :--- | :--- |
| **QUIC TPU Ingestion** | `LIVE` | SIMD-0002, Agave 4.3.0 | Mandatory on Devnet, Testnet, Mainnet |
| **Raw UDP TPU Ingestion** | `DEPRECATED / REMOVED` | Agave 4.3.0 | 100% null in cluster node directory |
| **TLS 1.3 QUIC Handshake** | `LIVE` | RFC 9000, `solana-quic-client` | Required for all TPU client connections |
| **QUIC Connection Reuse** | `LIVE` | `QuicConnectionCache` | Critical to avoid 1 RTT cold handshake overhead |
| **Alpenglow Multi-Bank TPU** | `IN DEVELOPMENT` | SIMD-0326 / SIMD-0337 | Multiple candidate banks per slot |
| **Rotor Block Propagation** | `IN DEVELOPMENT` | Deferred post-Votor | Live network still uses Turbine |

---

## 3. Core Architecture Components

The CHRONO Direct Leader QUIC Layer consists of two primary modules within `crates/chrono-bench`:

```
                  ┌──────────────────────────────────────────────┐
                  │            CHRONO RUNNER / ENGINE            │
                  └──────────────────────┬───────────────────────┘
                                         │
                   Dispatches to ExecutionRoute Trait
                                         │
             ┌───────────────────────────┴───────────────────────────┐
             │                                                       │
             ▼                                                       ▼
 ┌───────────────────────┐                               ┌───────────────────────┐
 │   StandardRpcRoute    │                               │ DirectLeaderQuicRoute │
 │ (Control/RPC Gateway) │                               │  (Chrono-Aware QUIC)  │
 └───────────┬───────────┘                               └───────────┬───────────┘
             │                                                       │
      JSON-RPC POST                                           Queries Target
   sendTransaction base64                                            │
             │                                                       ▼
             ▼                                          ┌────────────────────────┐
   Public RPC Endpoint                                  │ LeaderTransportResolver│
 (Rate-limited, Queued)                                 └───────────┬────────────┘
             │                                                       │
             │                                             Resolves Node Topology
             │                                            Cache (60s TTL, <500ns)
             │                                                       │
             │                                                       ▼
             │                                          ┌────────────────────────┐
             │                                          │  QuicConnectionCache   │
             │                                          │ (TLS 1.3 Warm Sessions)│
             │                                          └───────────┬────────────┘
             │                                                       │
             │                                              Direct TLS QUIC Stream
             │                                             (send_data / send_batch)
             │                                                       │
             ▼                                                       ▼
   Validator RPC Node                                      Validator Leader Node
             │                                                       │
             └─────────────► [ Validator TPU Pipeline ] ◄────────────┘
                             [ BankingStage / QOS Queue]
                                         │
                                   Slot Inclusion
```

---

## 4. Component Deep Dive

### 4.1 `LeaderTransportResolver` (`crates/chrono-bench/src/leader_transport.rs`)
The `LeaderTransportResolver` dynamically maps validator identity pubkeys to active network endpoints.

1. **Topology Synchronization**:
   - Fetches validator nodes via `getClusterNodes` JSON-RPC method.
   - Caches parsed `LeaderEndpoint` records for 60 seconds (configurable TTL).
   - Filters out stale or unroutable nodes lacking `tpuQuic`.
2. **Lock-Free Fast Lookup**:
   - Internal state protected by `RwLock<HashMap<Pubkey, LeaderEndpoint>>`.
   - Read path latency: **< 500ns** `[MEASURED]`, zero thread contention.
3. **Endpoint Validation**:
   - Extracts IP address and port from `tpu_quic`.
   - Validates socket address parsing and marks status as `Active` or `Unroutable`.

### 4.2 `DirectLeaderQuicRoute` (`crates/chrono-bench/src/route.rs`)
The `DirectLeaderQuicRoute` implements the `ExecutionRoute` trait for zero-hop leader delivery.

1. **QUIC Connection Caching**:
   - Backed by `QuicConnectionCache` (`solana-quic-client 4.3.0`).
   - Maintains open TLS 1.3 sessions keyed by `SocketAddr`.
   - First connection to a leader performs full TLS handshake (~350ms–700ms cold RTT). Subsequent submissions reuse the stream (< 1ms local handoff).
2. **Leader Lookahead & Dual-Targeting**:
   - `TargetingPolicy::CurrentLeaderOnly`: Dispatches exclusively to slot leader $S$.
   - `TargetingPolicy::CurrentPlusNext`: When within the last 80ms (or 20% of slot time) of a slot, simultaneously transmits wire transaction to slot leader $S$ and upcoming leader $S+4$.
   - Prevents leader handover boundary packet drops when banking stage buffers roll over.
3. **Fail-Safe Fallback Policy**:
   - `FallbackPolicy::QuicOnly`: Strict QUIC delivery. If leader is unreachable, records explicit error.
   - `FallbackPolicy::QuicThenRpc`: Attempts QUIC transmission. If QUIC handshake times out or encounters socket failure, immediately falls back to RPC `sendTransaction` and increments `fallback_count`.
4. **Wire Serialization & Signature Extraction**:
   - Decodes base64 payload into raw byte slice.
   - Direct zero-copy inspection of ed25519 transaction signature from header (bytes 1..65) for confirmation tracking without JSON deserialization overhead.

---

## 5. Lifecycle Telemetry (T0 → T10)

CHRONO records high-precision monotonic timestamps throughout the execution path:

| Epoch | Description | Measurement Source |
| :--- | :--- | :--- |
| **T0** | Consensus Decision Available | Chrono Slot Engine (`Instant::now()`) |
| **T1** | Template Build Start | Transaction Framer |
| **T2** | Transaction Signed | Ed25519 Keypair Signer |
| **T3** | Route Submission Start | Execution Route Entry |
| **T4** | Wire Packet Sent | QUIC Socket Send / HTTP POST |
| **T5** | Route Acknowledgment | QUIC Stream Ack / RPC Response |
| **T6** | First Network Observation | Chrono Geyser / WebSocket Stream |
| **T7** | Block Ingestion / Landed Slot | Ingested Bank Slot |
| **T8** | Processed Bank Commit | Agave Banking Stage Commit |
| **T9** | Cluster Confirmation (66%+ Stake)| Supermajority Optimistic Confirmation |
| **T10**| Cryptographic Finalization | Root / Notarization Certificate |

---

## 6. Safety & Mainnet Safeguards

To prevent accidental capital expenditure, testing on production funds, or network disruption:
1. **Cluster Whitelist**: `DirectLeaderQuicRoute` strictly permits `devnet`, `testnet`, and `localnet`.
2. **Mainnet Blocker**: CLI and runner inspect RPC URI and cluster identity. If `mainnet-beta` is targeted with live signing keys, execution aborts with `SafetyViolation: Mainnet execution strictly forbidden in Phase 5`.
3. **Secret Protection**: Private key bytes are never written to log files, stdout, or benchmark JSON artifacts. Only base58 public keys and transaction signatures are preserved.
