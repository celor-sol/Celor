# Solana Protocol & Alpenglow Specification

> **Specification Authority**: Solana Foundation SIMDs, Anza/Agave Releases, Helius & Triton Engineering Documentation.  
> **Classification Standard**: Every component is strictly classified as `LIVE`, `FEATURE-GATED`, `IN DEVELOPMENT`, `PROPOSED`, `FUTURE`, or `UNKNOWN`.

---

## 1. Overall Current Status
- **Status**: `IN DEVELOPMENT` (Devnet/Testnet progressive rollout; Mainnet pending)
- **Summary**: Alpenglow (governed primarily by SIMD-0326 and related SIMDs: SIMD-0337, SIMD-0298, SIMD-0525) represents a comprehensive consensus redesign of Solana. The core voting proposal passed governance in September 2025 with ~98.3% stake approval. Key components are introduced in Agave 4.2 / 4.3 releases and actively tested on Devnet and Testnet.
- **Empirical Testnet/Devnet Verification (Phase 1)**: `getAgGenesisCert` is confirmed active on both Solana Testnet and Devnet (`solana-core: 4.4.0-beta.0`), returning real 32-byte block IDs and 192-byte BLS aggregate signature certificates. Mainnet-Beta returns `null`, adhering strictly to legacy TowerBFT.
- **Primary Sources**: 
  - [SIMD-0326: Alpenglow Consensus](https://github.com/solana-foundation/solana-improvement-documents/blob/main/proposals/0326-alpenglow.md)
  - [SIMD-0337: Markers for Alpenglow Fast Leader Handover](https://github.com/solana-foundation/solana-improvement-documents)
  - [SIMD-0525: Slot Duration Reduction](https://github.com/solana-foundation/solana-improvement-documents)
  - Agave v4.3.0 Release Notes and Validator Documentation

---

## 2. Votor
- **Status**: `IN DEVELOPMENT` (Devnet/Testnet testing in Agave 4.3; not live on Mainnet)
- **Architecture**:
  - Replaces the legacy TowerBFT consensus engine and on-chain vote transactions.
  - Votes are communicated directly off-chain over dedicated validator network channels rather than submitted as on-chain transactions into blocks.
  - Frees approximately 70–75% of previous block space previously saturated with on-chain vote transactions.
  - Operates on a dual-path consensus model (Fast Path vs Fallback Path) designed around a **20+20 resilience model** (maintains safety and liveness with up to 20% Byzantine/adversarial stake and 20% offline/partitioned stake).
- **Source**: SIMD-0326, Agave 4.3 consensus documentation.

---

## 3. BLS Signatures & Aggregated Certificates
- **Status**: `IN DEVELOPMENT`
- **Architecture**:
  - Replaces per-validator Ed25519 signature verification on individual vote transactions with Boneh-Lynn-Shacham (BLS12-381) aggregate signatures.
  - Allows thousands of validator votes to be aggregated into a single compact cryptographic certificate (`block_final_cert`, `notar_reward_cert`, `skip_reward_cert`).
  - Greatly compresses state verification and block footer overhead.
- **Source**: SIMD-0326 § "Certificates and BLS Aggregation".

---

## 4. Finality Model: Fast Path vs Fallback Path
- **Status**: `IN DEVELOPMENT`
- **Mechanics**:
  - **Fast Path (Single-Round Notarization)**:
    - Threshold: **≥ 80% active stake**.
    - Round: Single round of notarization.
    - Deterministic Finality Latency: **~100ms** to **150ms**.
  - **Fallback Path (Two-Round Consensus)**:
    - Threshold: **≥ 60% active stake** (when 80% fast-path threshold is not achieved due to partitions or network delay).
    - Round: Two rounds of voting.
    - Deterministic Finality Latency: **~150ms** to **250ms**.
  - **Comparison with Legacy**: Legacy TowerBFT required 32 confirmations over consecutive slots with lockouts, yielding ~12.8s finality. Alpenglow achieves deterministic finality in a single slot window.
- **Source**: SIMD-0326 § "Consensus Paths and Latency Guarantees".

---

## 5. Slot Time Progression
- **Status**: 
  - 400ms → 350ms: `LIVE` (Mainnet August 2026, Epoch 1020)
  - 350ms → 300ms: `LIVE` (Mainnet August 2026, Epoch 1024)
  - 300ms → 250ms: `LIVE` (Mainnet September 18, 2026, Epoch 1037)
  - 250ms → 200ms: `FEATURE-GATED` (Tested on Devnet/Testnet; Mainnet activation pending stability observation)
- **Specification (SIMD-0525)**:
  - Each 50ms reduction is gated by a cluster feature gate.
  - Per-slot compute unit limits and target block limits are scaled down proportionally to keep total cluster compute throughput steady per wall-clock second while increasing responsiveness.
  - Applications relying on static `400ms` assumptions experience clock drift, premature transaction expiration calculations, and broken timeout logic.
- **Source**: SIMD-0525, Solana Release Communications, Helius Network Tracker.

---

## 6. Proof of History (PoH) & Ticks
- **Status**: `IN DEVELOPMENT` (Legacy PoH remains `LIVE` on Mainnet; Removal is active on Alpenglow test environments)
- **Impact**:
  - Under Alpenglow, PoH is **no longer a consensus primitive**.
  - **Ticks are removed from the ledger**: Block production is paced by Votor certificates and timeout timers rather than SHA-256 tick hash counts.
  - **What applications lose**:
    - Legacy intra-slot micro-timing estimation based on tick height is gone.
    - Systems that used tick counts to gauge progress within a 400ms block must switch to wall-clock timestamps and entry sequence counters.
- **Source**: SIMD-0326 § "De-coupling PoH and Ledger Structure".

---

## 7. Banks & Multi-Bank Slots
- **Status**: `IN DEVELOPMENT` (Agave 4.3 Devnet/Testnet)
- **Architecture**:
  - In legacy Solana, each slot strictly corresponded to a single linear bank sequence from parent to child.
  - Under Alpenglow's fast leader handoff, a leader can initiate block construction on candidate parent Bank A, but switch to Bank B if Bank B is finalized or notarized first.
  - Consequently, a single slot can produce **multiple candidate banks**.
  - Only one bank will be notarized and become canonical; abandoned banks are dropped.
- **Source**: Agave 4.3 Architecture Guide, Helius Engineering.

---

## 8. `bank_id`
- **Status**: `LIVE` in Agave 4.3+ client APIs; `IN DEVELOPMENT` on consensus level
- **Critical Invariant**:
  - `bank_id` is an internal `u64` identifier assigned by an Agave validator node to index local candidate bank instances.
  - **`bank_id` is strictly validator-local**: It has NO cluster-wide uniqueness guarantee. Two different validators (or two RPC providers like Helius vs Triton) will assign completely different `bank_id` values to the same block/bank.
  - **Cross-Provider Rule**: Infrastructure must buffer internally by `(provider_id, slot, bank_id)`, but reconcile globally across providers using `blockhash` or canonical block header identity.
- **Source**: Agave 4.3 Geyser API, Helius Technical Documentation.

---

## 9. UpdateParent
- **Status**: `IN DEVELOPMENT` (SIMD-0337, implemented in Agave 4.3)
- **Mechanics**:
  - Marker emitted in the shred stream (`BlockMarkerV1::UpdateParent`) when a leader changes the parent bank of the block currently under construction.
  - Notifies downstream validators, indexers, and Geyser plugins that previously streamed entries for the former parent bank belong to an abandoned bank candidate.
  - Consumer requirement: Must purge or tag uncommitted state for that `bank_id` to avoid processing stale or ghost transactions.
- **Source**: SIMD-0337, Agave 4.3 shred processing pipeline.

---

## 10. Fast Leader Handoff
- **Status**: `IN DEVELOPMENT`
- **Mechanics**:
  - Eliminates the idle "dead time" between successive leaders in the leader schedule.
  - Next leader can begin building shreds optimistically on the expected parent before full prior block completion, using `ParentReady` and `UpdateParent` signaling if the parent finalized differs from the optimistic branch.
  - Geographical latency implications: Minimizes transatlantic/transpacific leader transition penalties, dropping handoff delays from >100ms down to network flight time.
- **Source**: SIMD-0326, SIMD-0337.

---

## 11. Block Footer
- **Status**: `IN DEVELOPMENT` (Agave 4.3)
- **Structure**:
  - Appended at the end of block data (SIMD-0298 & Alpenglow).
  - Contains consensus metadata: `bank_hash` for execution state verification, and BLS consensus certificates (`block_final_cert`, `notar_reward_cert`, `skip_reward_cert`).
  - Cleanly decouples transaction payload execution from consensus proof storage.
- **Source**: SIMD-0298, Agave 4.3 Geyser plugin callback `notify_block_footer`.

---

## 12. Geyser & Yellowstone gRPC Updates
- **Status**: `IN DEVELOPMENT` (Agave 4.3 Proto definitions available; Yellowstone integration in progress)
- **New Callbacks & Fields**:
  - `notify_entry_update_parent`: Callback alerting plugins that an active block has switched parent, requiring invalidation of the abandoned `bank_id`.
  - `notify_block_footer`: Emits versioned block footer metadata containing finalization certificates and `bank_hash`.
  - Yellowstone protobufs: Addition of `bank_id` on transaction, account, and block-meta updates.
  - Migration Requirement: Downstream consumers must upgrade proto stubs and buffer data per `(slot, bank_id)`.
- **Source**: Triton One Yellowstone gRPC Repository, Agave Geyser SDK v4.3.0.

---

## 13. Finality Commitment Levels
- **Status**: `LIVE` (Legacy levels) / `IN DEVELOPMENT` (Alpenglow migration)
- **Commitment Comparison**:
  | Commitment | Legacy TowerBFT | Alpenglow / Votor | Recommended Application Action |
  |---|---|---|---|
  | **Observed / Processed** | Node has processed transaction in its local ledger | Node has ingested entry in local candidate bank | Use for optimistic speculation; check `UpdateParent` |
  | **Confirmed** | 1+ supermajority vote on slot (~400-800ms) | Deprecated in pure Alpenglow; maps to Fast Path Notarization | Transition downstream logic to Notarized / Finalized |
  | **Finalized** | 31+ lockouts (~12.8s) | BLS certificate attached (Fast Path ~100ms, Fallback ~150ms) | Treat as irrevocable deterministic settlement |
- **Source**: SIMD-0326 § "Commitment Levels and Deprecation Path".

---

## 14. Validator Economics: VAT (Validator Admission Ticket)
- **Status**: `PROPOSED` / `IN DEVELOPMENT`
- **Mechanics**:
  - Replaces the legacy vote transaction fee burning model (validators spent ~2-3 SOL/day submitting on-chain votes).
  - Introduces a flat Validator Admission Ticket (VAT), proposed at ~1.6 SOL per epoch, dynamically scaled by SIMD-0525 effective slot duration.
  - Decouples validator consensus participation from network fee mechanics.
- **Source**: SIMD-0326, SIMD-0525.

---

## 15. Rotor (Block Propagation)
- **Status**: `PROPOSED` / `IN DEVELOPMENT` (DEFERRED from initial Votor rollout; NOT live)
- **Architecture**:
  - Successor to the current **Turbine** tree-propagation protocol.
  - Uses stake-weighted direct distribution and erasure coding.
  - Incorporates **Smart Sampling** (inspired by FA1 committee sampling) to select relays with reduced variance.
  - **IMPORTANT OPERATIONAL NOTE**: Turbine remains the 100% active block propagation mechanism on all live Solana clusters today. CHRONO must NOT assume Rotor is active in initial implementations.
- **Source**: SIMD-0326 § "Rotor Propagation", Solana Core Team Updates.

---

## 16. Future Components (Smart Sampling & Asynchronous Execution)
- **Status**: `FUTURE` / `IN DEVELOPMENT`
  - **Smart Sampling**: Committee selection mechanism for Rotor block relayers to prevent eclipse attacks and bandwidth imbalances.
  - **Asynchronous Execution (Lazy Execution)**: Decoupling block scheduling from execution verification; foundational steps include Agave 3.0+ relaxed entry constraints and SIMD-0553 (Resource-Based Fees).
- **Source**: SIMD-0553, Anza Engineering Roadmap 2026.

---

## 17. FULL TELEMETRY ARCHITECTURE (Phase 1.5 Upgrade)

### 17.1 Telemetry Field Provenance Classification
To guarantee absolute data integrity and prevent hallucinated speed claims, every metric and field in Chrono Core carries an explicit provenance tag:

| Field Name | Standard RPC | Standard WS | Yellowstone gRPC | Local Geyser Mode | Derived Fallback Logic |
|---|---|---|---|---|---|
| **Slot** | `DIRECT` | `DIRECT` | `DIRECT` (`SubscribeUpdateSlot`) | `DIRECT` | Monotonic clock extrapolation |
| **Leader** | `DERIVED` (RPC schedule) | `UNAVAILABLE` | `DIRECT` | `DIRECT` | Derived from slot & epoch leader schedule |
| **Bank ID** | `UNAVAILABLE` | `UNAVAILABLE` | `DIRECT` (`SubscribeUpdateSlot.bank_id`) | `DIRECT` | `UNAVAILABLE` (Never fabricated; labeled `chrono-synth` if synthetic) |
| **Bank Hash** | `UNAVAILABLE` | `UNAVAILABLE` | `DIRECT` (`SubscribeUpdateBlockFooter.bank_hash`) | `DIRECT` | `UNAVAILABLE` (Strictly distinct from blockhash) |
| **Parent Lineage** | `DIRECT` (`parent_slot`) | `DIRECT` | `DIRECT` (`SubscribeUpdateBlock.parent_blockhash`) | `DIRECT` | Previous sealed blockhash |
| **UpdateParent** | `UNAVAILABLE` | `UNAVAILABLE` | `DIRECT` (`SubscribeUpdateEntryUpdateParent`) | `DIRECT` | `UNAVAILABLE` (Never hallucinated from fork variance) |
| **Block Footer** | `UNAVAILABLE` | `UNAVAILABLE` | `DIRECT` (`SubscribeUpdateBlockFooter`) | `DIRECT` | `UNAVAILABLE` |
| **Producer Time** | `UNAVAILABLE` | `UNAVAILABLE` | `DIRECT` (`block_producer_time_nanos`) | `DIRECT` | Monotonic observer arrival timestamp |
| **Producer User-Agent** | `UNAVAILABLE` | `UNAVAILABLE` | `DIRECT` (`block_user_agent`) | `DIRECT` | Validator client fingerprinting |
| **Final Certificate** | `UNAVAILABLE` | `UNAVAILABLE` | `DIRECT` (`block_final_cert` BLS bytes) | `DIRECT` | `DERIVED` from TowerBFT root confirmation |
| **Notar Reward Cert** | `UNAVAILABLE` | `UNAVAILABLE` | `DIRECT` (`notar_reward_cert` BLS bytes) | `DIRECT` | `UNAVAILABLE` |
| **Skip Reward Cert** | `UNAVAILABLE` | `UNAVAILABLE` | `DIRECT` (`skip_reward_cert` BLS bytes) | `DIRECT` | `UNAVAILABLE` |
| **Deshred Stream** | `UNAVAILABLE` | `UNAVAILABLE` | `DIRECT` (`SubscribeDeshred`) | `DIRECT` | `UNAVAILABLE` |

### 17.2 Cross-Provider Invariant: Bank-ID Scoping Standard
- **Within a single provider**: `(slot, bank_id)` is a valid validator-local identifier.
- **Across multiple providers**: `(slot, blockhash)` is the ONLY valid canonical reconciliation key.
- Two banks observed on different providers sharing the same numeric `bank_id` must NEVER be merged unless their `(slot, blockhash)` match identically.

### 17.3 Fast Leader Handover & Replacement Bank Sequence
Following an `UpdateParent` event:
1. `cleared_bank_id` is recorded and marked `BankState::Abandoned`. The node is NOT deleted.
2. Parent reference is redirected to `parent_block_id` or `parent_slot`.
3. When the replacement candidate bank is observed for that slot, `ReplacementEngine` correlates:
   `old bank (bank_id: A) -> cleared -> replacement bank (bank_id: B) -> canonical -> finalized`.
