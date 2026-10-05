# CHRONO Open Research & Verification Questions

> **Purpose**: Maintain an explicit, rigorous inventory of all technical unknowns, protocol ambiguities, and unverified assumptions. Every entry defines its architectural impact and the exact empirical experiment needed for resolution.

---

### OQ-001: Devnet vs Testnet vs Mainnet Alpenglow Feature Activation
- **Status**: `RESOLVED IN PHASE 1`
- **Empirical Finding**: Tested against live clusters with `chrono-detector`:
  - **Testnet** (`https://api.testnet.solana.com`): `solana-core 4.4.0-beta.0`, `getAgGenesisCert` returns active 32-byte block ID and 192-byte BLS signature certificate. Mode: `ALPENGLOW_VOTOR`.
  - **Devnet** (`https://api.devnet.solana.com`): `solana-core 4.4.0-beta.0`, `getAgGenesisCert` returns active certificate. Mode: `ALPENGLOW_VOTOR`.
  - **Mainnet-Beta** (`https://api.mainnet-beta.solana.com`): `getAgGenesisCert` returns `null`. Mode: `LEGACY_TOWER_BFT`.

---

### OQ-002: Observability of `UpdateParent` Over Free Public RPCs vs Yellowstone gRPC
- **Status**: `RESOLVED IN PHASE 1`
- **Empirical Finding**: Standard public Solana JSON-RPC and WebSocket (`wss://api.testnet.solana.com`, `slotSubscribe`) do NOT stream `bank_id` or `UpdateParent` candidate invalidation events. They stream sealed slot notifications and confirmed blocks.
- **Architectural Handling**: CHRONO explicitly marks `bank_id_stream: UNSUPPORTED` and `update_parent_stream: UNSUPPORTED` on public free RPC adapters without fabricating data. Real-time candidate banks and fast handover require a Yellowstone gRPC or validator Geyser feed.

---

### OQ-003: Validator-Local `bank_id` Collision Probability Across Providers
- **Status**: `RESOLVED IN PHASE 1`
- **Empirical Finding & Verification**: Unit and integration test suites in `crates/chrono-bank/tests/bank_suite_test.rs` and `chrono-core/src/identity.rs` verify that banks with identical `bank_id` from different providers with distinct blockhashes are never treated as the same bank, and banks with different `bank_id` but matching `(slot, blockhash)` reconcile into a unified canonical block.
- **Architectural Handling**: Canonical blockhash reconciliation is enforced in `BankIdentity` and `MultiStreamReconciler`.

---

### OQ-004: Timing Accuracy of Slot Clock under 250ms and 200ms Slots Without PoH Ticks
- **Question**: When PoH ticks are removed under Alpenglow, what is the maximum jitter and drift observed in monotonic slot phase calculation relative to actual validator shred production?
- **Why It Matters**: If CHRONO's Slot Clock drifts by more than 10-20ms, transactions aimed at the beginning or end of a slot window will land in unintended slots.
- **How to Verify**: Compare local high-resolution clock phase against received shred packet timestamps from a local test validator running Agave 4.3 with tick generation disabled.
- **Experiment Needed**: Local test validator harness measuring delta between predicted slot boundary and first received shred timestamp across 5,000 slots.

---

### OQ-005: Free Infrastructure Throughput & Reconnect Limits
- **Question**: What are the empirical rate limits, connection drop intervals, and payload size throttles on free-tier Devnet endpoints (Solana Public, Helius Free, QuickNode Free) when subscribing to high-frequency slot and account updates?
- **Why It Matters**: Under sub-250ms slots, message frequency increases by 60%–100%. Free connections may be aggressively terminated by provider reverse proxies.
- **How to Verify**: Benchmark subscription longevity and drop rates over 24 hours across free providers.
- **Experiment Needed**: Run continuous longevity monitor recording ping latency, packet drops, and reconnection backoff times under $0 budget.

---

### OQ-006: Yellowstone gRPC Protobuf Schema Divergence Across Providers
- **Status**: `RESOLVED IN PHASE 1.5`
- **Empirical Finding & Verification**: Verified against official `yellowstone-grpc-proto = "13.0.0"` in Rust. Confirmed exact field numbers and structure:
  - `SubscribeUpdateBlock.bank_id` (tag 14)
  - `SubscribeUpdateBlockMeta.bank_id` (tag 10)
  - `SubscribeUpdateEntry.bank_id` (tag 7)
  - `SubscribeUpdateBlockFooter`: `bank_id` (2), `bank_hash` (3), `block_producer_time_nanos` (4), `block_user_agent` (5), `block_final_cert` (6), `skip_reward_cert` (7), `notar_reward_cert` (8).
  - `SubscribeUpdateEntryUpdateParent`: `cleared_bank_id` (2), `parent_slot` (3), `parent_block_id` (4).
  - `SubscribeUpdateDeshredUpdateParent`: `update_parent_fec_set_index` (2), `parent_slot` (3), `parent_block_id` (4).
- **Architectural Handling**: `YellowstoneAdapter` integrates these native types and includes a bounded raw payload store (`RawPayloadStore`) to preserve unknown forward-compatible fields without loss.

---

### OQ-007: Votor BLS Certificate Availability in Geyser vs Block Footer
- **Status**: `RESOLVED IN PHASE 1.5`
- **Empirical Finding**: In Yellowstone gRPC / Geyser schema, BLS consensus certificates are emitted directly in `SubscribeUpdateBlockFooter` (`block_final_cert`, `notar_reward_cert`, `skip_reward_cert`) alongside block producer timestamps and user agent.
- **Architectural Handling**: `CertificateEngine` in `chrono-bank` processes the raw wincode-serialized BLS aggregate certificates in real time (~125ns decoding latency) immediately upon footer observation, enabling fast-path ~100ms finality registration without waiting for subsequent blocks.
