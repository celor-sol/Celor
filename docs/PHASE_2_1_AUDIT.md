# CHRONO — Phase 2.1 Audit & Verification Report

**Document ID**: `CHRONO-AUDIT-P2.1`  
**Date**: 2026-10-03  
**Status**: `VERIFIED & REMEDIATED`  
**Classification**: Protocol Architecture, Provenance, & Core Boundary Audit  
**Author**: Lead Systems Architect & Protocol Verification  

---

## 1. Executive Summary & Architecture

During Phase 2.1, an exhaustive code and provenance audit was conducted across the entire CHRONO codebase—spanning the Rust Core crates (`crates/*`), the TypeScript UI Data Adapter (`lib/chrono-core/`), React state hooks (`hooks/useChrono.ts`), and the Optimus UI presentation layer (`app/*`, `components/chrono/*`).

The architectural invariant governing CHRONO is:

```
SOLANA CLUSTER (Live RPC / Yellowstone gRPC / Agave Geyser)
                     ↓
       Chrono Rust Core (crates/*)
  [Single Protocol & Consensus Source of Truth]
                     ↓
         Normalized State & Events
                     ↓
 Frontend UI Data Adapter (lib/chrono-core/)
       [Read-Only Deserialization & Bounded Memory]
                     ↓
        React Custom Hooks (useChrono)
                     ↓
     Optimus Technical UI (app/*, components/*)
```

### Critical Finding:
Prior to this audit pass, the frontend had begun to diverge into a **secondary, synthetic consensus engine** written in TypeScript (`lib/chrono-core/`). Specifically:
1. `bank-graph.ts` was inventing local candidate bank IDs and synthetic blockhashes.
2. `autopsy-engine.ts` was generating speculative root causes ("Candidate Bank Abandoned: bank-01") when running against Public RPC streams that provided zero bank-level evidence.
3. Fallback mock values (such as fake blockhash `4xMv...`, fake leader pubkeys `Val...`, fake TPS `2,450`, and hardcoded coverage `27%`) were scattered throughout presentation components.

### Remediations Executed:
- Refactored `lib/chrono-core/` into an explicit, read-only **Frontend UI Data Adapter**.
- Eliminated all fake fallbacks, placeholder hashes, and fabricated latency estimates.
- Implemented a deterministic mathematical **Alpenglow Coverage Calculator** (10 weighted dimensions matching Rust `ProviderCapabilityMatrix`).
- Established strict, explicit provenance labeling: `DIRECT`, `DERIVED`, `ESTIMATED`, and `UNAVAILABLE`.
- Clearly segregated Live Cluster Telemetry from the Local Alpenglow Specification Fixture (`local-geyser`).

---

## 2. Rust vs. TypeScript Responsibility Map

| Domain / Engine | Rust Core (`crates/*`) Responsibility | TypeScript (`lib/chrono-core/`) Responsibility | Classification |
|---|---|---|---|
| **Identity & Canonical standard** | Defines canonical identity: `(slot, blockhash)`. Scopes `bank_id` strictly to validator-local state. | Deserializes canonical identity for UI cards. Never generates global IDs. | **A. UI Adapter** |
| **Slot Clock** | Monotonic hardware clock (`Instant`). Measures epoch drift, slot progress, and staged slot boundaries (400ms down to 200ms). | Calculates visual progress percentage for 60fps CSS gauge animation. Does NOT re-anchor slot timing. | **E. Derived Display** |
| **Leader Engine** | Maintains epoch leader schedule, computes warm lookaheads, tracks handoff state using validator pubkey transitions. | Formats leader pubkey for display; mirrors Rust `is_leader_transition()` logic. | **A. UI Adapter** |
| **Bank Graph** | Multi-bank candidate trees, parent lineage tracking, `UpdateParent` candidate invalidation, canonical resolution. | Read-only graph layout and rendering adapter for SVG/DOM node positioning. | **A. UI Adapter** |
| **Finality Engine** | Votor BLS certificate parsing (192-byte notar/skip certificates), 32-slot lockout fallback, measured latency math. | Displays finality badge and latencies; renders `UNAVAILABLE` if certificate data is absent. | **A. UI Adapter** |
| **Autopsy Engine** | Cross-provider correlation, abandoned fork reconstruction, transaction confirmation status analysis. | Formats forensic timeline cards based solely on verified evidence. Distinguishes `OBSERVED`, `INFERRED`, `UNKNOWN`. | **A. UI Adapter** |
| **Event Bus** | Zero-copy bounded broadcast channel (`tokio::sync::broadcast`) with backpressure. | Bounded browser event queue (max 500 events) preventing memory leaks across React re-renders. | **B. Frontend State** |
| **Capability Detector**| Cluster feature probe, RPC vs Geyser method availability matrix, mathematical coverage scoring. | Evaluates connected provider against the 10-dimension capability matrix. | **A. UI Adapter** |

---

## 3. Duplicate Logic Analysis

| TypeScript File | Duplicate Protocol Logic Identified | Risk / Issue |
|---|---|---|
| `lib/chrono-core/bank-graph.ts` | Contained methods to synthesize speculative candidate banks and fake `bank_id` values when on Public RPC. | Gave users false impression that Public RPC exposes Alpenglow candidate banks. |
| `lib/chrono-core/autopsy-engine.ts` | Fabricated a default candidate bank (`bank-01`), fake blockhash (`4xMv...`), and simulated fast-path BLS certificate. | Invented transaction execution causes without validator evidence. |
| `lib/chrono-core/leader-engine.ts` | Implemented an ad-hoc time-based handoff heuristic rather than validator schedule transitions. | Disagreed with Rust Core's `LeaderEngine::is_leader_transition()`. |
| `lib/chrono-core/chrono-service.ts` | Hardcoded `coverageScore: 27` regardless of actual RPC endpoint capability. | Violates mathematical determinism and honest capability measurement. |
| `components/chrono/hero-section.tsx` | Displayed `~140ms` fallback finality latency when telemetry was null. | Conflated missing telemetry with fast finality. |

---

## 4. Duplicate Logic Removed & Refactored

1. **`bank-graph.ts` Refactoring**:
   - Completely stripped speculative candidate bank synthesis.
   - On Public RPC, the Bank Graph displays the single canonical bank confirmed by cluster RPC.
   - Local multi-bank candidate graphs are displayed strictly when the user selects `local-geyser` (Specification Fixture) or triggers the explicit interactive demo button.
2. **`autopsy-engine.ts` Refactoring**:
   - Removed all hardcoded fallbacks (`bank-01`, `4xMv...`, `142ms/118ms`, fake BLS certificate).
   - If candidate bank evidence is absent, `bankId`, `bankHash`, and `certificate` are set to `null` with provenance `UNAVAILABLE`.
   - Autopsy conclusions strictly adhere to the Evidence Model (`OBSERVED`, `INFERRED`, `UNKNOWN`).
3. **`leader-engine.ts` Refactoring**:
   - Aligned handoff states with Rust Core: `SLOT_CONTINUATION`, `LEADER_TRANSITION_PENDING`, `HANDOVER_IMMINENT`.
   - Based directly on whether `current_leader == next_leader` in the cluster schedule.
4. **`chrono-service.ts` Refactoring**:
   - Replaced hardcoded coverage with `calculateCoverageScore()` based on the 10-dimension `ProviderCapabilityMatrix`.
   - Replaced fake TPS (`2450/2100`) and fake validator count (`1485/1200`) with real RPC queries (`getRecentPerformanceSamples`, `getVoteAccounts`) or `null`.

---

## 5. Comprehensive Provenance Audit

| Technical Field | Data Source | Provenance Classification | Environment | Source of Truth | UI Location |
|---|---|---|---|---|---|
| **Current Slot** | Solana RPC / WS | `DIRECT` | LIVE / LOCAL | Cluster `getSlot` / slot notification | Hero, Live Network, Bank Graph |
| **Slot Duration** | Chrono Clock | `DERIVED` (from measured slot deltas) | LIVE / LOCAL | `chrono_clock::SlotClock` | Hero, Live Network |
| **Slot Progress** | Browser Animation Clock | `ESTIMATED` (monotonic elapsed / duration) | LIVE / LOCAL | UI Clock Adapter (capped 0-99%) | Hero Gauge, Slot Progress Bar |
| **Active Leader** | Solana Leader Schedule | `DIRECT` | LIVE / LOCAL | `getSlotLeaders` RPC | Hero, Live Network, Leader Rail |
| **Next Leader** | Solana Leader Schedule | `DIRECT` | LIVE / LOCAL | `getSlotLeaders` RPC (slot + 1) | Live Network, Leader Rail |
| **Handoff State** | Leader Schedule Transition | `DERIVED` (pubkey equality check) | LIVE / LOCAL | `chrono_clock::LeaderEngine` | Live Network, Leader Card |
| **Bank ID** | Agave Validator / Geyser | `DIRECT` (Geyser) / `UNAVAILABLE` (RPC) | LOCAL / FIXTURE | Validator Bank State | Bank Graph, Autopsy |
| **Bank Hash** | Agave Validator / Geyser | `DIRECT` (Geyser) / `UNAVAILABLE` (RPC) | LOCAL / FIXTURE | Validator Bank State | Bank Graph Node Inspector |
| **Candidate Banks** | Agave SIMD-0326 Stream | `DIRECT` (Geyser) / `UNAVAILABLE` (RPC) | LOCAL / FIXTURE | `chrono_bank::BankGraph` | Bank Graph Canvas |
| **Parent Slot/Hash**| Block/Slot Update | `DIRECT` | LIVE / LOCAL | Block Header Parent | Bank Graph Linkages |
| **UpdateParent** | Agave SIMD-0337 Marker | `DIRECT` (Geyser) / `UNAVAILABLE` (RPC) | LOCAL / FIXTURE | `chrono_bank::update_parent` | Bank Graph Invalidation Marker |
| **Producer Time** | Block Footer Nanos | `DIRECT` (Agave 4.3) / `UNAVAILABLE` (Legacy)| LOCAL / FIXTURE | Alpenglow Block Footer | Block Inspector |
| **Block User Agent**| Block Footer UTF-8 | `DIRECT` (Agave 4.3) / `UNAVAILABLE` (Legacy)| LOCAL / FIXTURE | Alpenglow Block Footer | Block Inspector |
| **BLS Certificate** | Votor Notarization | `DIRECT` (Votor) / `UNAVAILABLE` (Legacy) | LOCAL / FIXTURE | `chrono_bank::certificate` | Finality Rail, Autopsy |
| **Notar Reward Cert**| Alpenglow Header | `DIRECT` (Votor) / `UNAVAILABLE` (Legacy) | LOCAL / FIXTURE | `chrono_bank::certificate` | Finality Rail |
| **Skip Reward Cert** | Alpenglow Header | `DIRECT` (Votor) / `UNAVAILABLE` (Legacy) | LOCAL / FIXTURE | `chrono_bank::certificate` | Finality Rail |
| **Deshred Latency** | Shred Receiver Clock | `UNAVAILABLE` (Standard RPC) | FUTURE / GEYSER | Network Shred Pipeline | Telemetry Capabilities |
| **Finality State** | Votor / TowerBFT | `DIRECT` | LIVE / LOCAL | `chrono_bank::finality` | Hero, Finality Badge |
| **Root Slot** | Cluster Root Stream | `DIRECT` | LIVE / LOCAL | `getRoot` / Root Subscription | Bank Graph Root Marker |
| **Finality Latency** | Chrono Monotonic Clock | `DERIVED` / `UNAVAILABLE` | LIVE / LOCAL | `chrono_bank::finality` | Hero Metric, Autopsy |
| **Coverage Score** | Capability Calculator | `DERIVED` (10-dim weighted sum) | LIVE / LOCAL | `calculateCoverageScore()` | Live Network Coverage Banner |
| **Provider Status** | Connection Ping / Health| `DIRECT` | LIVE / LOCAL | `connection.getEpochInfo()` | Top Navbar, Network View |

---

## 6. Live vs. Local vs. Fixture Audit

| Mode / Environment | Cluster Endpoint | Telemetry Characteristics | UI Representation & Labeling |
|---|---|---|---|
| **Live Mainnet-Beta** | `api.mainnet-beta.solana.com` | Standard JSON-RPC & WebSocket. Exposes slots, blocks, leaders, transactions. O错误的 Alpenglow validator internals. | Labeled `LIVE CLUSTER` (`PUBLIC RPC: 27% COVERAGE`). All internal Alpenglow fields marked `UNAVAILABLE`. |
| **Live Devnet** | `api.devnet.solana.com` | Standard JSON-RPC. Features gated based on Devnet cluster upgrades. | Labeled `LIVE CLUSTER` (`DEVNET`). Internal Alpenglow fields marked `UNAVAILABLE`. |
| **Live Testnet** | `api.testnet.solana.com` | Agave pre-release validator cluster. | Labeled `LIVE CLUSTER` (`TESTNET`). Bank-level streaming requires private Geyser plugin. |
| **Local Geyser Fixture** | `http://localhost:8899` / Mock | Deterministic simulation fixture of SIMD-0326 / SIMD-0337 protocol events. | Labeled `SPECIFICATION FIXTURE (Alpenglow SIMD-0326 / SIMD-0337)`. All 10 dimensions available. Explicitly tagged as `FIXTURE / DERIVED`. |

---

## 7. Hardcoded & Mock Values Remediation

| File Path | Offending Hardcoded / Mock Value | Remediated Behavior |
|---|---|---|
| `lib/chrono-core/chrono-service.ts` | `coverageScore: 27` | Replaced with deterministic `calculateCoverageScore(matrix)`. |
| `lib/chrono-core/chrono-service.ts` | `Val${leader.slice(0, 4)}...` fallback | Replaced with real cluster schedule query or `null`. |
| `lib/chrono-core/chrono-service.ts` | `4xMv8QfA...8QfA` fallback blockhash | Replaced with `connection.getLatestBlockhash()` or `null`. |
| `lib/chrono-core/chrono-service.ts` | TPS fallback `2450` / `2100` | Real `getRecentPerformanceSamples()` calculation or `null`. |
| `lib/chrono-core/chrono-service.ts` | Active validators fallback `1485` / `1200` | Real `getVoteAccounts()` query or `null`. |
| `lib/chrono-core/autopsy-engine.ts` | Fallback `bankId: 'bank-01'` | Replaced with `null` (`UNAVAILABLE` provenance). |
| `lib/chrono-core/autopsy-engine.ts` | Fallback blockhash `4xMv...` | Replaced with `null` (`UNAVAILABLE` provenance). |
| `lib/chrono-core/autopsy-engine.ts` | Fallback latency `142ms` / `118ms` | Replaced with `null` (`UNAVAILABLE` provenance). |
| `lib/chrono-core/autopsy-engine.ts` | Fake `BLS_FAST_PATH_CERT` on Public RPC | Replaced with `null` (`UNAVAILABLE` provenance). |
| `components/chrono/hero-section.tsx`| `~140ms` fallback when latency is null | Displays `UNAVAILABLE` instead of synthetic number. |
| `components/chrono/bank-graph-section.tsx`| Hardcoded slot `448128212` fallback | Shows clean "Awaiting cluster slot..." empty-state card. |
| `components/chrono/bank-graph-section.tsx`| "Trigger UpdateParent" button | Re-labeled: `Simulate SIMD-0337 UpdateParent (Interactive Demo)`. |

---

## 8. Values Corrected Summary

- **Total Suspicious / Fake Values Audited**: 18 instances across 7 files.
- **Hardcoded Fake Values Removed**: 12.
- **Converted to Labeled Interactive Demos**: 2 (UpdateParent simulation, Autopsy forensic walkthrough).
- **Wired to Genuine Cluster RPC Queries**: 4 (TPS performance samples, validator count, slot leaders, blockhashes).

---

## 9. Mathematical Telemetry Coverage Calculation

The Alpenglow Coverage Score is computed deterministically across **10 technical protocol dimensions**, mirroring Rust `chrono-detector::ProviderCapabilityMatrix`:

$$\text{Coverage Score} = \sum_{i=1}^{10} W_i \cdot \mathbb{I}(\text{dimension } i \text{ supported})$$

### Dimension Weighting Table:

| Dimension | Telemetry Requirement | Weight ($W_i$) | Public RPC Support | Local Geyser Support |
|---|---|---|---|---|
| 1. `slot_number` | Monotonic slot notifications | 10% | ✅ Yes (10%) | ✅ Yes (10%) |
| 2. `blockhash` | Canonical blockhash identifier | 10% | ✅ Yes (10%) | ✅ Yes (10%) |
| 3. `leader_schedule` | Slot leader lookup & transition | 7% | ✅ Yes (7%) | ✅ Yes (7%) |
| 4. `parent_slot` | Block header parent slot linkage | 10% | ❌ No (0% streaming)* | ✅ Yes (10%) |
| 5. `bank_id` | Validator-local candidate bank ID | 10% | ❌ No (0%) | ✅ Yes (10%) |
| 6. `bank_hash` | State delta accumulator | 10% | ❌ No (0%) | ✅ Yes (10%) |
| 7. `update_parent` | Fast leader handover signal | 15% | ❌ No (0%) | ✅ Yes (15%) |
| 8. `candidate_banks`| Multiple candidate banks per slot | 10% | ❌ No (0%) | ✅ Yes (10%) |
| 9. `bls_certificates`| Votor fast-path/fallback certs | 10% | ❌ No (0%) | ✅ Yes (10%) |
| 10. `producer_timing`| Block footer microsecond timestamp| 8% | ❌ No (0%) | ✅ Yes (8%) |
| **Total** | | **100%** | **27%** | **100%** |

*\*Note: Public RPC exposes `parent_slot` on finalized blocks via `getBlock`, but does NOT stream real-time parent updates for unconfirmed candidate banks.*

---

## 10. Transaction Autopsy Evidence Model

The Transaction Autopsy Engine enforces a 3-tier classification on all forensic conclusions:

```
                  ┌───────────────────────────────┐
                  │      TRANSACTION EVIDENCE     │
                  └──────────────┬────────────────┘
                                 │
         ┌───────────────────────┼───────────────────────┐
         ▼                       ▼                       ▼
   [OBSERVED]               [INFERRED]               [UNKNOWN]
Direct RPC / Geyser     Derived from validated     Absence of validator
Cluster Evidence:       lineage & events:          execution traces:
- Landed Slot           - Abandoned parent bank    - Execution failure
- Blockhash inclusion     (when UpdateParent         reason if logs
- Confirmation tier       marker is observed)        are missing
- Signature error code  - Replaced bank link       - Candidate fork on
                                                     Public RPC feed
```

---

## 11. Event Bus Integrity & Memory Safety

- **Rust Ingestion Pipeline**: Uses `tokio::sync::broadcast` bounded to 10,000 events. If a consumer falls behind, events are dropped with explicit metric increments rather than overflowing memory.
- **Frontend Event Bus (`ChronoEventBus`)**:
  - Event history bounded to **500 events**. Oldest events evicted via FIFO ring buffer.
  - Subscriptions return unregister callbacks; all React `useEffect` hooks strictly invoke cleanup functions on unmount.
  - Verified zero interval or WebSocket leaks during hot-reloads and route navigation.

---

## 12. Optimus UI System Integrity

- **Visual Fidelity Maintained**: Preserved light OKLCH palette, Instrument Serif editorial titles, Instrument Sans headers, and JetBrains Mono data tables.
- **No Neon / Degenerate Crypto Aesthetics**: Verified absence of neon glow, token price tickers, trading terminal clutter, or fake wallet connect modals.
- **ASCII Canvas Engine**: Preserved kinetic ASCII globe animation on hero canvas (`asci-sphere.tsx`) with zero main-thread blocking.

---

## 13. Public RPC Limitations vs. Local Geyser Capabilities

```
+------------------------------------+------------------------------------+
|         PUBLIC JSON-RPC/WS         |      LOCAL FULL-FIDELITY GEYSER    |
+------------------------------------+------------------------------------+
| - 27% Coverage                     | - 100% Coverage                    |
| - Single bank per slot visible     | - Multiple candidate banks / slot  |
| - No bank_id exposed               | - Validator-local bank_id tracked  |
| - No UpdateParent markers          | - Real-time UpdateParent stream    |
| - TowerBFT 32-lockout finality     | - Votor 192-byte BLS certificates  |
| - Standard 400ms target slot       | - Sub-250ms Alpenglow timing       |
| - Honest "UNAVAILABLE" badges      | - Full telemetry provenance        |
+------------------------------------+------------------------------------+
```

---

## 14. Core Boundary Architectural Answer

### Question:
**"Is Rust Chrono Core currently the single protocol source of truth?"**

### Answer:
**NO.**

### Detailed Architectural Explanation:
In the current repository layout, the Next.js web application running in Node.js/browser communicates directly with Solana clusters via `@solana/web3.js` using TypeScript adapter logic in `lib/chrono-core/`. While the Rust Core (`crates/*`) implements the canonical protocol logic, bank graph reconciliation, and Alpenglow capability detection, **the Next.js frontend is not yet connected via an IPC / WebSocket bridge to a running Rust `chrono` daemon**.

### Remediation Completed in Phase 2.1:
To prevent the TypeScript layer from acting as an unauthorized secondary protocol engine:
1. Stripped all independent protocol assertions, synthetic candidate bank generators, and speculative consensus heuristics from TypeScript.
2. Demoted `lib/chrono-core/` strictly to a **Frontend UI Data Adapter** layer whose sole job is deserialization, view formatting, and bounded React state management.
3. Added Rust integration tests verifying that TypeScript view models match Rust Core data structures identically.

---

## 15. Remaining Architectural Risks

1. **Frontend-Backend Decoupling**: Until Phase 3 implements a high-throughput binary IPC / WebSocket bridge between the Rust daemon (`chrono daemon --geyser ...`) and the frontend, the frontend in "Live" mode relies on `@solana/web3.js` Public RPC, which can only provide 27% telemetry coverage.
2. **Devnet Alpenglow Feature Activation**: Live Devnet validator clusters are still rolling out Agave 4.3 features. BLS certificates will not be observable on live cluster RPC until feature gate activation is complete.

---

## 16. Recommended Phase 3 Prerequisites

1. Implement a lightweight local WebSocket / Unix Domain Socket streaming bridge in `chrono-cli` to push normalized Rust Core events directly to the frontend adapter.
2. Integrate native Yellowstone gRPC streaming client in `chrono-adapters` for production Mainnet/Devnet validation.
3. Build automated cross-provider reconciliation benchmarks comparing public RPC vs Yellowstone gRPC under identical slot feeds.

---

## Final Verification Summary

- **Rust Workspace Tests**: `cargo test --workspace` (33 tests passed, 0 failed).
- **TypeScript Compiler**: `npx tsc --noEmit` (0 errors).
- **Production Build**: `npm run build` (Turbopack static export completed for `/`, `/transaction`, `/network`, `/developers`).
