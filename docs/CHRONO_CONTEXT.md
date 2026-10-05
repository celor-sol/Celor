# CHRONO Project Context

## 1. What CHRONO Is

**CHRONO** is a provider-independent consensus timing, bank graph, and stream reconciliation infrastructure layer engineered for Solana's modern and future architecture. It bridges low-latency streaming infrastructure (Yellowstone gRPC, Geyser plugins, WebSocket RPC) with consensus-aware execution primitives, explicitly accounting for Alpenglow (SIMD-0326), sub-250ms slot progression (SIMD-0525), validator-local `bank_id` handling, fast leader handovers (`UpdateParent` / SIMD-0337), and fast finality notarization.

---

## 2. Why CHRONO Exists

Solana is transitioning from its legacy architecture (400ms target slots, TowerBFT on-chain voting with ~12.8s finality, synchronous entry constraints) into a high-frequency, certificate-based protocol:
1. **Dynamic Slot Durations**: Slot target reduction from 400ms down through 350ms, 300ms, 250ms (active on Mainnet Epoch 1037), targeting 200ms (SIMD-0525).
2. **Multi-Bank Slots & Fast Leader Handover**: Leaders switch parent banks via `UpdateParent` markers, producing multiple candidate banks per slot. Naive indexers and trading systems that key exclusively by `slot` face silent state corruption, stale data processing, and transaction drops.
3. **Provider Fragmentation & Local Scoping**: Key identifiers such as `bank_id` (introduced in Agave 4.3) are **validator-local** and cannot be compared directly across different RPC or Geyser providers.
4. **Vendor Lock-in**: Developers and searchers are fragmented across proprietary APIs (e.g., proprietary gRPC wrappers, closed execution networks) with no unified, open-source infrastructure capable of multi-provider stream reconciliation.

CHRONO resolves these problems by providing an open, unified, protocol-accurate timing engine, bank graph tracker, and canonical resolver.

---

## 3. Target Users

- **Quantitative Traders & MEV Searchers**: Needing microsecond-accurate leader schedule tracking, bank fork resolution, and zero-stale-state transaction injection.
- **DeFi Protocols & Liquidators**: Requiring sub-150ms finality detection, safe oracle updates, and accurate liquidation trigger timing.
- **Indexers & Data Pipelines**: Needing safe ingestion across multiple banks per slot, handling `notify_entry_update_parent` and block footers without corrupted account states.
- **RPC & Infrastructure Operators**: Needing vendor-neutral stream aggregation and health monitoring across diverse RPC/Geyser upstreams.

---

## 4. Current $0 Budget Constraint

CHRONO is bootstrapped under an absolute **$0 operational budget**:
- **Free/Open Infrastructure**: Development, profiling, and validation must use public Devnet/Testnet endpoints, free-tier provider offerings (Helius free tier, Triton/Yellowstone test endpoints, QuickNode free tier, public Solana RPC nodes), and local test validators (`solana-test-validator`).
- **No Paid Clusters or Relays**: No paid dedicated Geyser nodes, private fiber (DoubleZero), or enterprise gRPC tiers will be assumed or required during initial phases.
- **Zero-Cost High Performance**: Optimization must derive from architectural efficiency, clean zero-allocation data structures, and accurate protocol logic, not raw paid bandwidth.

---

## 5. Phased Progression: Devnet/Testnet to Mainnet

1. **Bootstrap Phase (Current)**: Protocol research, architecture specification, benchmark protocol definition, decision logs, and agent rules. Zero application code.
2. **Phase 1 (Devnet/Testnet Verification)**:
   - Local and Devnet cluster adapters.
   - Slot clock and protocol feature detector.
   - Leader engine tracking epoch schedules and geographical handoffs.
   - Multi-provider stream adapter (Public RPC + Free Yellowstone gRPC).
3. **Phase 2 (Bank Graph & Reconciliation)**:
   - `(slot, bank_id)` tuple buffering.
   - `UpdateParent` bank invalidation and pruning.
   - Canonical bank resolution via block identity/blockhash.
4. **Phase 3 (Finality Engine & Event Bus)**:
   - Votor certificate ingestion (`block_final_cert`, `notar_reward_cert`) vs TowerBFT confirmation tracking.
   - Unified high-performance event bus for consumers.
5. **Phase 4 (Mainnet Production & Execution Routing)**:
   - Multi-provider Mainnet reconciliation (Helius, Triton, Yellowstone, local nodes).
   - Low-latency transaction routing and leader-aware injection.

---

## 6. What CHRONO Is NOT

- **CHRONO is NOT an out-of-the-box memecoin sniper bot**: It is high-performance consensus and timing infrastructure upon which execution strategies can be built.
- **CHRONO is NOT a proprietary closed-source RPC service**: It is an open, provider-independent software layer.
- **CHRONO is NOT a validator client**: It does not produce blocks or participate in consensus voting; it ingests, tracks, normalizes, and acts upon consensus streams.
- **CHRONO does NOT assume unreleased features are live**: It strictly separates live protocol mechanisms (e.g., Turbine, 250ms slots) from in-development proposals (e.g., Rotor, Asynchronous Execution).

---

## 7. Current Project Status

- **Status**: Research, Architecture Specification, and Customization Bootstrap complete.
- **Codebase**: Repository cleanly initialized; architectural rules established; zero production application code written. Ready for Phase 1 planning and execution.
