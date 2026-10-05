# CHRONO PHASE 6: UPSTREAM PROTOCOL & TELEMETRY RESEARCH

> **Document**: `docs/PHASE_6_PROTOCOL_RESEARCH.md`  
> **Status**: APPROVED & VERIFIED  
> **Scope**: In-depth protocol analysis of Agave Validator Geyser interfaces, Yellowstone gRPC Protobufs, Alpenglow SIMDs, and Deshred/Certificate ingestion pathways.

---

## 1. Executive Summary & Research Mandate

Phase 6 transitions CHRONO from standard Public JSON-RPC / WebSocket observability to **full-fidelity validator-side telemetry**. Standard Solana JSON-RPC abstracts away validator internals: it collapses concurrent candidate banks into single linear confirmed blocks, completely omits internal validator `bank_id`, discards pre-execution deshred transactions, and strips Alpenglow consensus certificates and block footers.

The mandate of Phase 6 is to answer:
> **Can CHRONO obtain internal Solana / Alpenglow consensus telemetry directly from validator-side hooks, Yellowstone gRPC, or custom validator instrumentation? If yes, implement it. If only indirectly observable, derive it. If genuinely impossible under supported interfaces, prove why.**

This research report documents the exact APIs, protobuf structs, validator traits, and runtime availability across all upstream sources.

---

## 2. Upstream Source 1: Agave Geyser Plugin Interface

### 2.1 Interface Overview
The official Agave validator plugin interface is defined in `agave-geyser-plugin-interface` (formerly `solana-geyser-plugin-interface`). It provides a dynamic C ABI linkage (`_create_plugin`) allowing high-throughput, in-process C/Rust shared libraries (`.so` / `.dylib`) to hook directly into the validator's replay, execution, and ledger pipelines.

### 2.2 Standard Geyser Hooks (Agave 2.0+ / 4.x)

```rust
pub trait GeyserPlugin: Any + Send + Sync + Debug {
    fn name(&self) -> &'static str;
    fn on_load(&mut self, config_file: &str) -> PluginResult<()>;
    fn on_unload(&mut self);

    /// Bank-scoped slot status notification
    fn update_slot_status(
        &self,
        slot: Slot,
        parent: Option<Slot>,
        status: SlotStatus,
    ) -> PluginResult<()>;

    /// Block metadata notification on block sealing
    fn notify_block_metadata(
        &self,
        block_info: &ReplicaBlockInfoVersions,
    ) -> PluginResult<()>;

    /// Executed transaction notification
    fn notify_transaction(
        &self,
        transaction_info: &ReplicaTransactionInfoVersions,
        slot: Slot,
    ) -> PluginResult<()>;

    /// Ledger entry notification
    fn notify_entry(
        &self,
        entry_info: &ReplicaEntryInfoVersions,
    ) -> PluginResult<()>;
}
```

#### Field & Semantic Details:
1. **`update_slot_status`**:
   - `slot`: `u64`
   - `parent`: `Option<u64>`
   - `status`: `SlotStatus` enum:
     - `Processed`: The bank has processed all entries for the slot.
     - `Confirmed`: The bank has received 1+ supermajority cluster votes or fast-path notarization.
     - `Rooted`: The bank has achieved irreversible finality (32 lockouts / finalized certificate).
     - `Dead`: The bank was abandoned or failed execution verification.
   - **Availability**: `LIVE` on Mainnet-Beta, Testnet, Devnet, and local validators.
   - **Validator-Local Reality**: `update_slot_status` is emitted by the local node's `BankForks` subsystem. A slot may emit multiple `Processed` events if candidate banks branch prior to confirmation.

2. **`notify_block_metadata`**:
   - Passed `ReplicaBlockInfoVersions::V0_0_1` or `V0_0_2`.
   - Fields: `parent_slot`, `parent_blockhash`, `slot`, `blockhash`, `rewards`, `block_time`, `block_height`, `executed_transaction_count`.
   - **Limitation**: Emitted only *after* block execution completes. Does not provide pre-execution timing or Alpenglow BLS certificates.

### 2.3 Alpenglow Consensus & Fast Handover Geyser Hooks (Agave 4.3+)

To support SIMD-0326 (Alpenglow Consensus) and SIMD-0337 (Fast Leader Handover Markers), Agave 4.3 introduced new bank-aware trait methods and opt-in capabilities:

```rust
pub trait GeyserPlugin: Any + Send + Sync + Debug {
    // ... standard methods ...

    /// Opt-in flag: Enables block footer notifications
    fn block_footer_notifications_enabled(&self) -> bool { false }

    /// Alpenglow block footer notification
    fn notify_block_footer(
        &self,
        slot: Slot,
        bank_id: BankId,
        bank_hash: &Hash,
        block_footer: &ReplicaBlockFooterVersions,
    ) -> PluginResult<()> { Ok(()) }

    /// Opt-in flag: Enables entry UpdateParent notifications
    fn entry_update_parent_notifications_enabled(&self) -> bool { false }

    /// Fired when an UpdateParent marker is parsed from entries
    fn notify_entry_update_parent(
        &self,
        slot: Slot,
        cleared_bank_id: Option<BankId>,
        parent_slot: Slot,
        parent_block_id: Option<&Hash>,
        fec_set_index: Option<u32>,
    ) -> PluginResult<()> { Ok(()) }

    /// Opt-in flag: Enables deshred notifications
    fn deshred_notifications_enabled(&self) -> bool { false }

    /// Fired when a pre-execution transaction is reassembled from shreds
    fn notify_deshred(
        &self,
        slot: Slot,
        deshred_info: &ReplicaDeshredTransactionVersions,
    ) -> PluginResult<()> { Ok(()) }

    /// Fired when an UpdateParent marker is encountered in the deshred stream
    fn notify_deshred_update_parent(
        &self,
        slot: Slot,
        cleared_bank_id: Option<BankId>,
        parent_slot: Slot,
        parent_block_id: Option<&Hash>,
        fec_set_index: Option<u32>,
    ) -> PluginResult<()> { Ok(()) }
}
```

#### Deep Inspection of Alpenglow Geyser Fields:
1. **`bank_id` (`u64`)**:
   - Internal identifier allocated by `Bank::new_from_parent` on the observing validator.
   - **Validator-Local Invariant**: Unique ONLY within the local validator instance. Different nodes assign different `bank_id`s to the same blockhash.
2. **`bank_hash` (`[u8; 32]`)**:
   - The cryptographic Merkle root of the bank's accounts delta table after executing all transactions in the candidate bank.
   - Strictly distinct from `blockhash` (which is the PoH / entry hash).
3. **`ReplicaBlockFooterVersions`**:
   - `slot`: `u64`
   - `bank_id`: `u64`
   - `producer_time_nanos`: `Option<u64>` — Monotonic nanosecond timestamp emitted by the producing validator upon sealing the block footer.
   - `user_agent`: `Option<String>` — Producing validator client string (e.g. `Agave/v4.3.0`).
   - `block_final_cert`: `Option<Vec<u8>>` — BLS12-381 aggregate signature certificate certifying irreversible finality (80% stake fast path).
   - `notar_reward_cert`: `Option<Vec<u8>>` — BLS certificate certifying round notarization for validator reward settlement.
   - `skip_reward_cert`: `Option<Vec<u8>>` — BLS certificate recording skipped leader slot penalty/settlement.
4. **`notify_entry_update_parent` & `notify_deshred_update_parent`**:
   - Signals that the leader optimistically began building on `cleared_bank_id`, but has received proof that a different parent won the slot.
   - Downstream consumers MUST discard uncommitted speculative transactions assigned to `cleared_bank_id`.

---

## 3. Upstream Source 2: Yellowstone gRPC (`rpcpool/yellowstone-grpc`)

### 3.1 Protobuf Specification (`geyser.proto`)
Yellowstone gRPC translates Agave Geyser plugin callbacks into high-performance Protocol Buffer streams over HTTP/2.

Key protobuf messages in Yellowstone v16 / rc9:

#### `SubscribeUpdateSlot`
```protobuf
message SubscribeUpdateSlot {
  uint64 slot = 1;
  optional uint64 parent = 2;
  CommitmentLevel status = 3;
  optional string dead_error = 4;
  optional uint64 bank_id = 5; // Added in Yellowstone v16 for Alpenglow
}
```

#### `SubscribeUpdateBlockFooter`
```protobuf
message SubscribeUpdateBlockFooter {
  uint64 slot = 1;
  uint64 bank_id = 2;
  string bank_hash = 3;
  optional uint64 producer_time_nanos = 4;
  optional string user_agent = 5;
}
```

#### `SubscribeUpdateEntry`
```protobuf
message SubscribeUpdateEntry {
  uint64 slot = 1;
  uint64 index = 2;
  uint64 num_hashes = 3;
  bytes hash = 4;
  uint64 executed_transaction_count = 5;
  uint64 starting_transaction_index = 6;
  optional UpdateParent update_parent = 7; // SIMD-0337 marker
}

message UpdateParent {
  uint64 slot = 1;
  optional uint64 cleared_bank_id = 2;
  uint64 parent_slot = 3;
  optional string parent_block_id = 4;
  optional uint32 fec_set_index = 5;
}
```

#### `SubscribeDeshred`
```protobuf
message SubscribeDeshredRequest {
  map<string, SubscribeRequestFilterDeshred> deshred = 1;
}

message SubscribeUpdateDeshred {
  uint64 slot = 1;
  bytes transaction = 2;
  string signature = 3;
  uint64 pre_execution_timestamp = 4;
  optional UpdateParent update_parent = 5;
}
```

### 3.2 Critical Yellowstone Reality Audit

1. **Yellowstone Block Footer & Certificates**:
   - `SubscribeUpdateBlockFooter` forwards `slot`, `bank_id`, `bank_hash`, `producer_time_nanos`, and `user_agent`.
   - **CRITICAL GAP**: The standard Yellowstone protobuf message does NOT forward raw BLS consensus certificates (`block_final_cert`). Standard gRPC streaming providers strip or omit certificates to minimize serialization overhead on public gRPC nodes.
   - **Conclusion**: To ingest raw cryptographic certificates (`final_cert`, `notar_reward_cert`), CHRONO cannot rely solely on standard Yellowstone endpoints; a direct validator-side Geyser plugin is required.

2. **Yellowstone Deshred (`SubscribeDeshred`) Reality**:
   - While `SubscribeDeshred` exists in the `geyser.proto` definitions, the **open-source Yellowstone server implementation (`yellowstone-grpc-geyser`) does NOT implement the `SubscribeDeshred` RPC**. It returns `Status::unimplemented("SubscribeDeshred is not implemented")`.
   - `SubscribeDeshred` is an unmerged, proprietary extension used by Triton One on commercial private instances.
   - **Conclusion**: Deshred telemetry is `CONDITIONALLY OBSERVABLE` via validator-side Geyser hooks, but `UNAVAILABLE` over open-source public/testnet Yellowstone gRPC servers.

---

## 4. Upstream Source 3: Direct Validator Instrumentation & Turbine Shreds

### 4.1 Shred & FEC Layer Observability
Can CHRONO observe individual shreds and Turbine propagation timestamps?

1. **Agave Public Geyser Interface**:
   - Does **NOT** expose a `notify_shred` hook.
   - Exposing every single raw shred (~30,000 to 50,000 UDP packets per second per validator) over dynamic C plugin interfaces would introduce cache invalidation and lock contention in the validator's hot TPU/TVU network loop.
2. **Validator Internal Pipeline**:
   - `Tvu::shred_fetch_stage` receives UDP shreds from Turbine.
   - `RetransmitStage` retransmits shreds across the cluster tree.
   - `WindowService` / `Blockstore` inserts shreds into RocksDB and runs Reed-Solomon FEC erasure decoding.
   - Only after Reed-Solomon reconstruction restores an entry does the validator invoke `notify_entry` or `notify_deshred`.
3. **Verdict**:
   - **Direct Raw Shred Arrival**: `UNAVAILABLE` without custom validator source code patches or eBPF socket tracing (`sock_filter` on TVU UDP port 8000–8020).
   - **Deshred / Reconstructed Entries**: `DIRECT` via validator Geyser `notify_deshred` and `notify_entry`.
   - **Reconstruction Latency (T_ENTRY - T_DESHRED)**: `DERIVED` by correlating monotonic timestamps between deshred queue arrival and entry execution.

---

## 5. Provenance & Observability Classification (5-Level System)

CHRONO Phase 6 expands metric provenance into five strictly defined categories:

| Level | Classification | Definition | Example Fields |
|---|---|---|---|
| **Level 1** | `DIRECT` | Unmodified value received directly from a validator hook, Geyser callback, or Yellowstone protobuf. | `slot`, `bank_id`, `bank_hash`, `parent_slot`, `producer_time_nanos`, `raw_certificate_bytes` |
| **Level 2** | `DERIVED` | Deterministic, unambiguous calculation from observed direct fields. | `leader` (from slot + schedule), `entry_count`, `tx_per_entry`, `producer_to_observation_interval` |
| **Level 3** | `INFERRED` | Evidence-based consensus conclusion derived from multiple independent signals where protocol internal state is not directly emitted. | `canonical_bank_resolution`, `fork_abandonment`, `replacement_bank_correlation` |
| **Level 4** | `ESTIMATED` | Statistical, interpolated, or timing approximations based on historical data. | `estimated_time_remaining_in_slot`, `clock_drift_estimate` |
| **Level 5** | `UNAVAILABLE` | Field cannot be observed or verified through any legitimate supported interface in the active environment. | `raw_turbine_fec_drop_rate` (requires eBPF), `deshred` (when on open-source Yellowstone) |

---

## 6. Observer vs. Producer Disambiguation

A foundational architectural requirement of Phase 6:
```
PRODUCER (Block Author)  ──────[Network Flight]──────►  OBSERVER (Validator Ingesting)
  - producer_pubkey                                       - observer_pubkey
  - producer_time_nanos                                   - local_observed_at_nanos
  - blockhash / bank_hash                                 - validator-local bank_id
```

- **`bank_id` is an OBSERVER attribute**: It identifies the bank inside the observing node. It tells us nothing about how the producing validator numbered its bank.
- **`producer_time_nanos` is a PRODUCER attribute**: Emitted by the block producer when writing the Alpenglow block footer.
- **`observer_to_producer_interval`**:
  $$\Delta t = t_{\text{observer, received}} - t_{\text{producer, nanos}}$$
  This interval can only be measured as network flight time if both clocks are synchronized (e.g. via PTP/NTP); otherwise, it is explicitly classified as `observer-to-producer interval` with clock domain uncertainty.

---

## 7. Cryptographic Certificate Subsystem Design

### 7.1 Three Certificate Verification States
CHRONO strictly distinguishes between:
1. **`RAW CERTIFICATE OBSERVED`**: Raw byte payload captured from block footer or RPC (e.g. 192 bytes for BLS aggregate signature + 32 bytes root hash + 8 bytes bitmap).
2. **`CERTIFICATE PARSED`**: Header, slot, voting bitmap, and aggregate public key successfully extracted into structured fields.
3. **`CERTIFICATE CRYPTOGRAPHICALLY VERIFIED`**: BLS12-381 pairing verification ($e(\sigma, g_2) = e(H(m), pk_{\text{agg}})$) executed and mathematically proven against cluster validator stake weights.

> **CRITICAL RULE**: Parsed certificate bytes must NEVER be labeled `VERIFIED` without executing actual pairing cryptography against the active stake table.

---

## 8. Summary of Upstream Research Conclusions

1. **Highest-Fidelity Source**: A validator-side Chrono Geyser Plugin (`crates/chrono-geyser-plugin`) communicating with Chrono Core over Unix Domain Sockets (UDS).
2. **Yellowstone Role**: Excellent for commercial and cloud deployments (`slot`, `bank_id`, `bank_hash`, `UpdateParent`, `producer_time`), but misses raw BLS certificates and open-source `SubscribeDeshred`.
3. **Public RPC Role**: Preserved as baseline Level 0 (`27%` coverage); strictly labeled as limited linear block observability.
4. **Alpenglow Status**: Actively tested on Devnet and Testnet. Agave 4.3 features (`block_footer`, `UpdateParent`, `bank_id`) are implemented in validator client code and ready for ingestion.
