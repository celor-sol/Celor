# CHRONO Phase 4 Baseline & Competitor Research

> **Document**: `docs/PHASE_4_BASELINE.md`  
> **Status**: Verified Engineering Baseline  
> **Objective**: Document empirical baseline capabilities of contemporary Solana execution stacks to isolate exactly what Chrono adds.

---

## 1. Executive Summary

A core directive of CHRONO Phase 4 is: **Never assume competitors are slow or that Chrono is faster.**

Modern Solana execution systems already utilize highly optimized components:
- Direct TPU QUIC streaming
- Jito Block Engine bundles
- Dedicated shred-stream relays (Yellowstone gRPC, Geyser plugins)
- Pre-warmed connection pools and pre-flight simulation skips

The objective of this research is to establish what a standard low-latency Solana execution stack can achieve today, establish empirical baselines across standard network conditions, and delineate the boundary where Chrono provides distinct structural value versus where Chrono offers no material latency advantage.

---

## 2. Contemporary Execution Mechanisms & Baseline Latencies

| Execution Architecture | Submission Mechanism | Landing Mechanism | Observed Confirmation Latency (p50) | Inherent Limitations & Edge-Case Failure Modes |
|---|---|---|---|---|
| **Standard Public JSON-RPC** | HTTP POST `/sendTransaction` (`skipPreflight: true`) | Validator RPC forwards transaction to cluster leader over TPU | **1,200ms – 2,400ms** | Buffer bloat, leader-unaware round-trips, stale blockhash errors during cluster stalls, high drop rate (>20% under congestion). |
| **Optimized Direct TPU (e.g. Solana Rust Client / TPU Client)** | Direct UDP/QUIC packet stream to current validator leader TPU port | Ingested directly into leader BankingStage packet queue | **350ms – 650ms** | Sensitive to leader window handoff timing. Packets sent during the last 50ms of a 4-slot leader window frequently miss the leader or arrive out of order at the subsequent leader. |
| **Jito Block Engine (MEV / Private Relay)** | Tip-paying bundle submission to Jito Block Engine relayer endpoint | Ingested via private block-engine pipeline to Jito-Solana validator | **400ms – 800ms** (or next Jito leader slot) | Dependent on Jito leader schedule (not all validators run Jito-Solana). Requires payment of tip lamports ($ > 0). Does not address validator-internal candidate bank invalidation. |
| **Yellowstone gRPC / Geyser Streamer** | High-velocity streaming subscription; paired with TPU forwarder | Direct slot and transaction update broadcast via protobuf | **Sub-50ms observation** | Observability only. Does not provide execution routing decisions, parent invalidation logic, or autonomous stale-blockhash avoidance. |

---

## 3. The Structural Gaps in Contemporary Stacks

While existing execution bots and private relays minimize physical wire latency (using Rust, QUIC, and colocation), they treat consensus as a black box:

### Gap 1: Blind Submissions Across Leader Transitions
- Standard clients submit transactions without considering remaining slot tenure or leader handoff state.
- In a 400ms (or SIMD-0326 staged 250ms/200ms) slot, transactions dispatched in the final 10–25% of the window experience up to a 4-slot delay (+1.6s) if dropped by the transitioning leader's BankingStage.

### Gap 2: Stale Blockhash Expiry Under Stall
- Conventional clients obtain a recent blockhash on an ad-hoc basis (fetching `/getLatestBlockhash` upon transaction intent).
- If network RTT fluctuates or cluster block production stalls, the blockhash expires before landing, causing silent drops or explicit `BlockhashNotFound` errors.

### Gap 3: Total Ignorance of Candidate Banks (Alpenglow / SIMD-0326)
- In the Alpenglow architecture, multiple candidate banks can exist per slot.
- Existing trading clients and RPCs assume `slot == block` and treat any observed slot state as canonical.
- If a candidate bank is subsequently abandoned via an `UpdateParent` consensus marker, any transaction routed or assumed executed against that bank's state becomes invalid.

---

## 4. What Chrono Adds (And What It Does NOT Add)

### What Chrono Adds:
1. **Continuous Slot & Leader Tenure Telemetry**: Microsecond-accurate tracking of elapsed slot percentage and imminent leader transition, enabling execution systems to make deterministic `SUBMIT`, `WAIT`, or `RETRY` decisions.
2. **Synchronized Blockhash Freshness Model**: Pre-warmed, stream-maintained blockhash state with multi-tier freshness classification (`Fresh`, `Aging`, `Stale`, `Expired`), preventing obsolete submissions.
3. **Candidate Bank & UpdateParent Invalidation Engine**: Dynamic tracking of candidate banks, parent lineage switches, and fast BLS finality certificates, preventing execution systems from acting on orphaned forks.
4. **Deterministic Routing State Machine**: Explainable infrastructure decisions that correlate consensus signals with transaction outcomes.

### What Chrono Does NOT Add:
1. **Magic Network Acceleration**: Chrono does not bypass the laws of physics. If an external RPC or validator network link has 120ms RTT, Chrono cannot reduce that transport delay.
2. **Guaranteed Landing on Unfunded Accounts**: Chrono cannot force a cluster to process transactions that fail fee payment or runtime simulation checks.
3. **Artificial Speed Claims**: Chrono maintains strict benchmarking integrity; microsecond CPU decision times are never marketed as network landing latencies.
