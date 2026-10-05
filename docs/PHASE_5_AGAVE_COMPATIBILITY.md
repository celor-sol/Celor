# CHRONO — Phase 5 Agave & TPU Client Compatibility Audit

> **Document**: `docs/PHASE_5_AGAVE_COMPATIBILITY.md`  
> **Status**: Verified Operational Specification  
> **Date**: October 4, 2026  
> **Workspace Toolchain**: Rust 1.99.0 (b940084d7 2026-09-28)  

---

## 1. Crate Dependency Audit

The CHRONO workspace compiles against the official Agave 4.3+ ecosystem releases:

| Crate | Version | Status | Role in Phase 5 |
|---|---|---|---|
| `solana-client` | `4.3.0` | Active / Merged | Cluster JSON-RPC, `getClusterNodes`, `getSlot`, `getSignatureStatuses` |
| `solana-tpu-client` | `4.3.0` | Active / Stable | Established Agave TPU client with `solana-connection-cache` |
| `solana-tpu-client-next` | `4.3.0` | Experimental / Unstable | Next-generation stream-based TPU client (Agave 4.3+ modularization) |
| `solana-connection-cache` | `4.3.0` | Active / Stable | QUIC connection pool, TLS 1.3 session resumption, connection reuse |
| `solana-pubkey` | `4.4.0` | Active / Stable | Base58 32-byte Ed25519 validator identity representation |
| `quinn` | `0.11` | Active / Transitive | Underlying asynchronous IETF QUIC transport implementation |
| `rustls` | `0.23` | Active / Transitive | Modern TLS 1.3 engine for QUIC handshaking and certificate verification |

---

## 2. Agave Transport Reality: QUIC vs UDP Ingestion

### Deprecation of UDP Transaction Ports
In historic Solana releases (prior to v1.14 / v2.0), validators exposed raw UDP sockets on their TPU port for unthrottled transaction ingestion. This architecture was vulnerable to sybil packet flooding and IP spoofing.

Under current Agave architecture (v2.0 through v4.3+ / Alpenglow):
- **UDP transaction ingestion is strictly disabled/removed** on public mainnet-beta, testnet, and devnet clusters.
- All validator nodes expose a **TPU QUIC port** (typically `TPU_PORT + 6` or negotiated via `getClusterNodes`).
- Ingestion enforces TLS 1.3 authentication, connection limits, and per-stake packet bandwidth throttling.
- **Directive**: CHRONO Phase 5 **does NOT implement raw UDP sockets**. All direct leader transport is QUIC-native.

---

## 3. TPU Client Architecture Evaluation

The Agave 4.3 tree provides two distinct TPU client interfaces:

### Option A: `solana-tpu-client` (Established Standard)
- **Features**:
  - Encapsulates `ConnectionCache` supporting QUIC connection pooling.
  - Subscribes to cluster leader schedule via JSON-RPC WebSocket.
  - Handles packet serialization, connection eviction, and backpressure.
  - Fully compatible with `solana-client 4.3.0`.
- **Verdict**: Selected as the proven, production-grade Agave TPU client foundation.

### Option B: `solana-tpu-client-next` (Agave Modular Stream Architecture)
- **Features**:
  - Independent async streaming client decoupled from monolithic RPC dependencies.
  - Interfaces directly with `quinn` connections and `solana-packet`.
  - Requires unstable custom TLS certificate generation and raw packet pinning.
- **Verdict**: Marked as `UNSTABLE / IN DEVELOPMENT`. We document its design in architecture notes but avoid relying on its non-finalized traits for primary benchmark paths.

---

## 4. CHRONO Direct Leader QUIC Layer Design

Rather than treating the TPU client as an opaque black box, CHRONO implements an explicit **four-component architecture**:

1. **`LeaderTransportResolver`**:
   - Resolves target validator pubkey $\to$ IP:Port endpoint using cached `getClusterNodes` cluster gossip data.
   - Enforces TTL-based cache invalidation and tracks endpoint freshness.
2. **`DirectLeaderQuicRoute`**:
   - Implements the `ExecutionRoute` trait.
   - Manages QUIC connection lifecycle (cold connect, warm reuse, handshake timing).
   - Distinguishes **transport handoff** (local enqueue) from **on-chain landing** (independent verification).
3. **Execution State Machine & Fallback Engine**:
   - Configurable fallback modes: `QUIC_ONLY`, `QUIC_THEN_RPC`, and `RPC_ONLY`.
   - Never silently falls back during pure scientific experiments.
4. **Independent On-Chain Verifier**:
   - Cross-checks transaction signature statuses using independent JSON-RPC polling to verify true landing slots and confirmation latencies.
