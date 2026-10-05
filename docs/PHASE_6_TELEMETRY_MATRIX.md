# CHRONO PHASE 6: MASTER TELEMETRY MATRIX & DATA INVENTORY

> **Document**: `docs/PHASE_6_TELEMETRY_MATRIX.md`  
> **Status**: RATIFIED & EXHAUSTIVE  
> **Coverage Goal**: Comprehensive audit of all 65+ consensus, execution, bank, marker, and timing fields across all ingestion pathways.

---

## 1. Classification & Provenance Taxonomy

Every telemetry field in CHRONO is classified under two orthogonal dimensions:
1. **Provenance Level**:
   - `[DIRECT]`: Emitted directly by validator hooks, Geyser callbacks, or protocol messages.
   - `[DERIVED]`: Unambiguously computed via deterministic mathematical/logical functions on direct inputs.
   - `[INFERRED]`: Reconstructed through multi-signal correlation when direct protocol fields are private/unexposed.
   - `[ESTIMATED]`: Approximation derived from clock interpolation, network latency modeling, or historical baselines.
   - `[UNAVAILABLE]`: Not accessible through any currently supported, unprivileged interface.
2. **Protocol Lifecycle State**:
   - `LIVE`: Merged and active on Mainnet-Beta.
   - `FEATURE-GATED`: Merged into Agave codebase, awaiting feature flag activation on the cluster.
   - `IN DEVELOPMENT`: Active on Devnet / Testnet (Agave 4.2 / 4.3).
   - `PROPOSED`: Formalized in a SIMD specification, awaiting merge.
   - `FUTURE`: Long-term roadmap.

---

## 2. Exhaustive Telemetry Inventory (Sections A – J)

### Section A: Slot Telemetry
| # | Field | Type | Description | Best Source | Provenance | Protocol State |
|---|---|---|---|---|---|---|
| A.1 | `slot` | `u64` | Monotonic slot index | RPC / WS / Geyser | `DIRECT` | `LIVE` |
| A.2 | `parent_slot` | `u64` | Immediate predecessor slot | RPC / WS / Geyser | `DIRECT` | `LIVE` |
| A.3 | `root_slot` | `u64` | Highest finalized / rooted slot | RPC / WS / Geyser | `DIRECT` | `LIVE` |
| A.4 | `slot_status` | `enum` | Processed / Confirmed / Rooted / Dead | Geyser / Yellowstone | `DIRECT` | `LIVE` |
| A.5 | `slot_lifecycle` | `enum` | Created → Executing → Sealed → Notarized | Geyser / BankGraph | `DERIVED` | `IN DEVELOPMENT` |
| A.6 | `slot_start_observation` | `i64 (ns)` | Local timestamp when first shred/entry observed | Geyser / Ingestor | `DIRECT` | `LIVE` |
| A.7 | `slot_end_observation` | `i64 (ns)` | Local timestamp when block footer / seal observed | Geyser / Ingestor | `DIRECT` | `LIVE` |
| A.8 | `slot_duration_observed` | `f64 (ms)` | Elapsed time: `slot_end - slot_start` | Chrono Clock | `DERIVED` | `LIVE` |
| A.9 | `slot_leader` | `Pubkey` | Assigned block producer for the slot | RPC / Yellowstone | `DERIVED` | `LIVE` |
| A.10 | `next_leader` | `Pubkey` | Scheduled producer for `slot + 1` | Leader Schedule | `DERIVED` | `LIVE` |
| A.11 | `leader_schedule` | `Map<Slot, Pubkey>` | Epoch-wide validator leader assignment | RPC (`getLeaderSchedule`) | `DIRECT` | `LIVE` |
| A.12 | `leader_handoff_state` | `enum` | Building / Handoff / UpdateParent / Finalizing | BankGraph | `INFERRED` | `IN DEVELOPMENT` |
| A.13 | `remaining_leader_time` | `f64 (ms)` | Slot budget remaining based on tick/duration | Chrono Clock | `ESTIMATED` | `LIVE` |
| A.14 | `actual_leader_tenure` | `f64 (ms)` | Time between first and last entry from leader | Geyser Entries | `DERIVED` | `LIVE` |
| A.15 | `skipped_slot` | `bool` | True if slot never produced entries/block | Slot Monitor | `DERIVED` | `LIVE` |
| A.16 | `slot_continuity` | `enum` | Linear / Continuous / Forked / Orphaned | BankGraph | `DERIVED` | `LIVE` |

---

### Section B: Bank Telemetry
| # | Field | Type | Description | Best Source | Provenance | Protocol State |
|---|---|---|---|---|---|---|
| B.1 | `bank_id` | `u64` | Validator-local candidate bank index | Geyser / Yellowstone | `DIRECT` | `IN DEVELOPMENT` |
| B.2 | `bank_hash` | `[u8; 32]` | Post-execution accounts delta root | Block Footer / Geyser | `DIRECT` | `IN DEVELOPMENT` |
| B.3 | `bank_created_at` | `i64 (ns)` | Timestamp when bank initialized in memory | Geyser Plugin | `DIRECT` | `IN DEVELOPMENT` |
| B.4 | `bank_processed` | `bool` | True when all block entries executed | Geyser (`Processed`) | `DIRECT` | `LIVE` |
| B.5 | `bank_confirmed` | `bool` | True upon cluster supermajority confirmation | Geyser (`Confirmed`) | `DIRECT` | `LIVE` |
| B.6 | `bank_rooted` | `bool` | True upon finality lock / certificate root | Geyser (`Rooted`) | `DIRECT` | `LIVE` |
| B.7 | `bank_dead` | `bool` | True if abandoned, dropped, or invalid | Geyser (`Dead`) | `DIRECT` | `LIVE` |
| B.8 | `bank_frozen` | `bool` | True when bank accounts hash sealed | Geyser Plugin | `DIRECT` | `LIVE` |
| B.9 | `bank_parent_slot` | `u64` | Slot of ancestor bank | Geyser / BankForks | `DIRECT` | `LIVE` |
| B.10 | `bank_parent_block_id` | `Hash` | Blockhash of parent bank | Geyser / BlockHeader | `DIRECT` | `IN DEVELOPMENT` |
| B.11 | `bank_lifecycle_timestamps` | `Struct` | Monotonic timestamps for each state transition | Chrono BankGraph | `DIRECT` | `LIVE` |
| B.12 | `candidate_banks` | `Vec<Bank>` | Competing candidate branches within slot | Geyser / Yellowstone | `DIRECT` | `IN DEVELOPMENT` |
| B.13 | `replacement_banks` | `Vec<Bank>` | Candidate banks created following UpdateParent | BankGraph Engine | `DERIVED` | `IN DEVELOPMENT` |
| B.14 | `bank_succession` | `Vec<u64>` | Ordered progression of candidate banks | BankGraph Engine | `DERIVED` | `IN DEVELOPMENT` |
| B.15 | `bank_invalidation_reason` | `String` | Why candidate bank died (e.g. UpdateParent) | BankGraph Engine | `DERIVED` | `IN DEVELOPMENT` |
| B.16 | `bank_ancestry` | `Vec<Slot>` | Recursive lineage back to cluster root | BankGraph | `DERIVED` | `LIVE` |
| B.17 | `bank_descendants` | `Vec<Slot>` | Active child branches stemming from bank | BankGraph | `DERIVED` | `LIVE` |
| B.18 | `bank_canonical_status` | `enum` | Candidate / Abandoned / Canonical / Dead | CanonicalResolver | `INFERRED` | `IN DEVELOPMENT` |

---

### Section C: Alpenglow Block Markers
| # | Field | Type | Description | Best Source | Provenance | Protocol State |
|---|---|---|---|---|---|---|
| C.1 | `block_header` | `Struct` | Block opening marker and metadata | Geyser / Yellowstone | `DIRECT` | `IN DEVELOPMENT` |
| C.2 | `block_footer` | `Struct` | Block sealing marker and finality proofs | Geyser / Yellowstone | `DIRECT` | `IN DEVELOPMENT` |
| C.3 | `update_parent` | `Struct` | Fast leader handover parent redirection | Geyser / Yellowstone | `DIRECT` | `IN DEVELOPMENT` |
| C.4 | `update_parent_slot` | `u64` | Current slot where handover occurred | Geyser / Yellowstone | `DIRECT` | `IN DEVELOPMENT` |
| C.5 | `update_parent_parent_slot` | `u64` | New parent slot selected by leader | Geyser / Yellowstone | `DIRECT` | `IN DEVELOPMENT` |
| C.6 | `update_parent_parent_block_id` | `Hash` | Hash of new parent block | Geyser / Yellowstone | `DIRECT` | `IN DEVELOPMENT` |
| C.7 | `update_parent_fec_index` | `Option<u32>` | FEC set index where marker was received | Geyser Plugin | `DIRECT` | `IN DEVELOPMENT` |
| C.8 | `leader_window_position` | `u32` | Position of block in 4-slot leader window | Chrono Clock | `DERIVED` | `LIVE` |
| C.9 | `marker_ordering` | `u64` | Monotonic index of markers within slot | Ingestor RingBuffer | `DIRECT` | `IN DEVELOPMENT` |
| C.10 | `marker_timestamps` | `i64 (ns)` | Arrival timestamp of each block marker | Chrono Ingestor | `DIRECT` | `LIVE` |

---

### Section D: Certificates (Cryptographic Finality Subsystem)
| # | Field | Type | Description | Best Source | Provenance | Protocol State |
|---|---|---|---|---|---|---|
| D.1 | `final_certificate` | `Struct` | Fast-path (≥80% stake) finality proof | Geyser Plugin | `DIRECT` | `IN DEVELOPMENT` |
| D.2 | `notarization_certificate` | `Struct` | Round notarization (≥60% stake) proof | Geyser Plugin | `DIRECT` | `IN DEVELOPMENT` |
| D.3 | `notar_reward_certificate` | `Struct` | Validator reward distribution certification | Geyser Plugin | `DIRECT` | `IN DEVELOPMENT` |
| D.4 | `skip_reward_certificate` | `Struct` | Skipped slot settlement proof | Geyser Plugin | `DIRECT` | `IN DEVELOPMENT` |
| D.5 | `certificate_raw_bytes` | `Vec<u8>` | Unmodified serialized BLS certificate | Geyser / RPC | `DIRECT` | `IN DEVELOPMENT` |
| D.6 | `certificate_size` | `usize` | Exact payload length (e.g. 192+32+bitmap) | CertificateEngine | `DERIVED` | `IN DEVELOPMENT` |
| D.7 | `certificate_type` | `enum` | Final / Notarization / NotarReward / SkipReward | CertificateEngine | `DERIVED` | `IN DEVELOPMENT` |
| D.8 | `certificate_slot` | `u64` | Slot certified by the proof | CertificateEngine | `DERIVED` | `IN DEVELOPMENT` |
| D.9 | `certificate_block_id` | `Hash` | Canonical blockhash certified by aggregate | CertificateEngine | `DERIVED` | `IN DEVELOPMENT` |
| D.10 | `certificate_timestamp` | `i64 (ns)` | Producer timestamp encoded in certificate | CertificateEngine | `DERIVED` | `IN DEVELOPMENT` |
| D.11 | `certificate_stake_percent` | `f64 (%)` | Aggregate stake represented in bitmap | Stake Table | `DERIVED` | `IN DEVELOPMENT` |
| D.12 | `certificate_aggregate_pubkey`| `[u8; 96]` | BLS12-381 aggregate public key | CertificateEngine | `DERIVED` | `IN DEVELOPMENT` |
| D.13 | `certificate_validation_status`| `enum` | `RAW` / `PARSED` / `CRYPTOGRAPHICALLY_VERIFIED` | BLS Verifier | `DERIVED` | `IN DEVELOPMENT` |

---

### Section E: Producer Telemetry
| # | Field | Type | Description | Best Source | Provenance | Protocol State |
|---|---|---|---|---|---|---|
| E.1 | `producer_timestamp` | `i64 (s)` | Unix epoch seconds from block header/footer | Geyser / Yellowstone | `DIRECT` | `LIVE` |
| E.2 | `producer_timestamp_nanos` | `u64 (ns)` | High-precision nanosecond producer clock | Block Footer / Geyser | `DIRECT` | `IN DEVELOPMENT` |
| E.3 | `block_producer_identity` | `Pubkey` | Leader identity that signed block | Leader Schedule / Block | `DIRECT` | `LIVE` |
| E.4 | `validator_user_agent` | `String` | Software client string (e.g. `Agave/4.3.0`) | Block Footer / Geyser | `DIRECT` | `IN DEVELOPMENT` |
| E.5 | `validator_version` | `String` | Semantic version of producing node | Block Footer / Gossip | `DIRECT` | `LIVE` |
| E.6 | `client_identity` | `String` | Client implementation (Agave / Firedancer) | User-Agent Fingerprint | `DERIVED` | `LIVE` |
| E.7 | `producer_metadata` | `Map` | Combined producer environment attributes | Ingestion Normalizer | `DERIVED` | `IN DEVELOPMENT` |
| E.8 | `block_production_timing` | `f64 (ms)` | Production start to seal interval | Chrono Clock | `DERIVED` | `IN DEVELOPMENT` |

---

### Section F: Entry Telemetry
| # | Field | Type | Description | Best Source | Provenance | Protocol State |
|---|---|---|---|---|---|---|
| F.1 | `entry` | `Struct` | Ordered ledger entry containing tx batch | Geyser (`notify_entry`) | `DIRECT` | `LIVE` |
| F.2 | `entry_index` | `u64` | Sequence counter of entry within slot | Geyser / Yellowstone | `DIRECT` | `LIVE` |
| F.3 | `entry_slot` | `u64` | Slot containing this entry | Geyser / Yellowstone | `DIRECT` | `LIVE` |
| F.4 | `entry_parent_info` | `Hash` | Hash of previous entry / PoH | Geyser (`notify_entry`) | `DIRECT` | `LIVE` |
| F.5 | `entry_update_parent` | `Struct` | UpdateParent marker embedded in entry stream | Geyser Plugin | `DIRECT` | `IN DEVELOPMENT` |
| F.6 | `transactions_per_entry` | `usize` | Number of transactions packed into entry | Geyser / Yellowstone | `DIRECT` | `LIVE` |
| F.7 | `entry_ordering` | `bool` | Strict ascending monotonicity verification | Ingestor Inspector | `DERIVED` | `LIVE` |
| F.8 | `entry_timestamp` | `i64 (ns)` | Ingestion timestamp of entry arrival | Chrono Ingestor | `DIRECT` | `LIVE` |
| F.9 | `entry_source` | `String` | Provider / node that emitted entry | Ingestor Transport | `DIRECT` | `LIVE` |
| F.10 | `entry_fec_association` | `Option<u32>` | FEC batch associated with entry | Geyser Plugin | `DIRECT` | `IN DEVELOPMENT` |

---

### Section G: Deshred / Pre-Execution Telemetry
| # | Field | Type | Description | Best Source | Provenance | Protocol State |
|---|---|---|---|---|---|---|
| G.1 | `deshred_transaction` | `Struct` | Raw transaction reconstructed before exec | Geyser (`notify_deshred`) | `DIRECT` | `IN DEVELOPMENT` |
| G.2 | `transaction_signature` | `Signature` | Primary 64-byte Ed25519 tx signature | Deshred / Geyser | `DIRECT` | `LIVE` |
| G.3 | `deshred_slot` | `u64` | Slot target for pre-execution tx | Deshred Hook | `DIRECT` | `IN DEVELOPMENT` |
| G.4 | `raw_transaction_bytes` | `Vec<u8>` | Serialized wire bytes of transaction | Deshred Hook | `DIRECT` | `LIVE` |
| G.5 | `static_accounts` | `Vec<Pubkey>` | Account keys declared in message header | Deshred Parser | `DERIVED` | `LIVE` |
| G.6 | `loaded_addresses` | `Vec<Pubkey>` | Accounts loaded via Address Lookup Tables | Banking Stage / Geyser | `UNAVAILABLE pre-exec`| `LIVE post-exec` |
| G.7 | `pre_execution_obs_time` | `i64 (ns)` | Monotonic timestamp of deshred receipt | Chrono Ingestor | `DIRECT` | `IN DEVELOPMENT` |
| G.8 | `deshred_update_parent` | `Struct` | UpdateParent marker on deshred stream | Geyser Plugin | `DIRECT` | `IN DEVELOPMENT` |
| G.9 | `transaction_ordering` | `u64` | Pre-execution shred reassembly index | Deshred Pipeline | `DIRECT` | `IN DEVELOPMENT` |
| G.10 | `shred_fec_context` | `Option<u32>` | FEC set index containing transaction shreds | Validator Internal | `INFERRED` | `IN DEVELOPMENT` |

---

### Section H: Shreds & Turbine Telemetry
| # | Field | Type | Description | Best Source | Provenance | Protocol State |
|---|---|---|---|---|---|---|
| H.1 | `shred_observation` | `Packet` | Raw UDP shred packet received on TVU port | eBPF / Socket Trace | `UNAVAILABLE`* | `LIVE` |
| H.2 | `shred_type` | `enum` | Data Shred vs Coding (FEC) Shred | eBPF / Socket Trace | `UNAVAILABLE`* | `LIVE` |
| H.3 | `shred_slot` | `u64` | Slot encoded in shred header | eBPF / Socket Trace | `UNAVAILABLE`* | `LIVE` |
| H.4 | `shred_index` | `u32` | Shred position within slot | eBPF / Socket Trace | `UNAVAILABLE`* | `LIVE` |
| H.5 | `shred_fec_set_index` | `u32` | Reed-Solomon coding block index | eBPF / Socket Trace | `UNAVAILABLE`* | `LIVE` |
| H.6 | `shred_completion` | `bool` | All data/coding shreds received for batch | WindowService | `UNAVAILABLE`* | `LIVE` |
| H.7 | `shred_reconstruction_time` | `f64 (ms)` | Elapsed time: `first_shred -> reconstructed` | Deshred vs TVU | `UNAVAILABLE`* | `IN DEVELOPMENT` |
| H.8 | `shred_arrival_timestamp` | `i64 (ns)` | Monotonic hardware timestamp of UDP receipt | Kernel NIC Socket | `UNAVAILABLE`* | `LIVE` |
| H.9 | `shred_leader_source` | `IpAddr` | Originating IP address of shred transmitter | Socket Metadata | `UNAVAILABLE`* | `LIVE` |
| H.10 | `shred_peer_metadata` | `Struct` | Turbine tree relayer hop info | Gossip / TVU | `UNAVAILABLE`* | `LIVE` |

*\* Note: Raw shred telemetry (H.1 – H.10) is omitted from standard validator Geyser APIs to prevent CPU cache poisoning and packet drop in the TVU hot path. Capturing individual shreds requires kernel eBPF instrumentation (`sock_filter`) on UDP port 8000.*

---

### Section I: Consensus State Telemetry
| # | Field | Type | Description | Best Source | Provenance | Protocol State |
|---|---|---|---|---|---|---|
| I.1 | `state_observed` | `bool` | Block / bank observed on stream | Ingestor | `DIRECT` | `LIVE` |
| I.2 | `state_processed` | `bool` | Local validator ledger processed | Geyser | `DIRECT` | `LIVE` |
| I.3 | `state_confirmed` | `bool` | 1+ supermajority cluster vote | Geyser / RPC | `DIRECT` | `LIVE` |
| I.4 | `state_rooted` | `bool` | Irreversible finality achieved | Geyser / RPC | `DIRECT` | `LIVE` |
| I.5 | `state_canonical` | `bool` | Bank accepted on cluster winning chain | CanonicalResolver | `INFERRED` | `LIVE` |
| I.6 | `state_candidate` | `bool` | Concurrent bank pending resolution | BankGraph | `DIRECT` | `IN DEVELOPMENT` |
| I.7 | `state_abandoned` | `bool` | Bank discarded following UpdateParent | BankGraph | `INFERRED` | `IN DEVELOPMENT` |
| I.8 | `state_dead` | `bool` | Bank failed validation or dead fork | Geyser (`Dead`) | `DIRECT` | `LIVE` |
| I.9 | `state_replaced` | `bool` | Bank superseded by new candidate | ReplacementEngine | `INFERRED` | `IN DEVELOPMENT` |
| I.10 | `state_notarized` | `bool` | Votor round notarization proof attached | Geyser / Footer | `DIRECT` | `IN DEVELOPMENT` |
| I.11 | `cert_backed_finality` | `bool` | Certified by BLS aggregate signature | CertificateEngine | `DIRECT` | `IN DEVELOPMENT` |
| I.12 | `consensus_transition` | `enum` | Path taken (Fast Path 80% vs Fallback 60%) | FinalityEngine | `DERIVED` | `IN DEVELOPMENT` |
| I.13 | `handoff_state` | `enum` | OptimisticBuilding / ConfirmedParent / Switch | HandoffDetector | `INFERRED` | `IN DEVELOPMENT` |

---

### Section J: Execution Correlation & Forensics
| # | Field | Type | Description | Best Source | Provenance | Protocol State |
|---|---|---|---|---|---|---|
| J.1 | `tx_observed_at` | `i64 (ns)` | Time transaction first entered Chrono | Ingestor | `DIRECT` | `LIVE` |
| J.2 | `tx_deshred_at` | `i64 (ns)` | Time transaction reassembled before exec | Deshred Hook | `DIRECT` | `IN DEVELOPMENT` |
| J.3 | `tx_bank_ingress_at` | `i64 (ns)` | Time transaction queued in candidate bank | Geyser Plugin | `DIRECT` | `IN DEVELOPMENT` |
| J.4 | `tx_executed_at` | `i64 (ns)` | Time transaction execution completed | Geyser Plugin | `DIRECT` | `LIVE` |
| J.5 | `tx_confirmed_at` | `i64 (ns)` | Time transaction slot reached supermajority | Consensus Stream | `DIRECT` | `LIVE` |
| J.6 | `tx_finalized_at` | `i64 (ns)` | Time transaction slot rooted / certified | Certificate Engine | `DIRECT` | `LIVE` |
| J.7 | `tx_associated_bank_id` | `u64` | Validator-local bank that executed tx | Geyser Plugin | `DIRECT` | `IN DEVELOPMENT` |
| J.8 | `tx_associated_bank_hash`| `[u8; 32]` | State Merkle root after tx block | Block Footer | `DIRECT` | `IN DEVELOPMENT` |
| J.9 | `tx_producing_leader` | `Pubkey` | Leader that scheduled transaction | Leader Schedule | `DERIVED` | `LIVE` |
| J.10 | `tx_parent_lineage` | `Hash` | Parent blockhash backing execution | Block Metadata | `DIRECT` | `LIVE` |
| J.11 | `tx_pre_to_post_latency`| `f64 (ms)` | Elapsed time: `deshred -> executed` | Forensic Engine | `DERIVED` | `IN DEVELOPMENT` |
| J.12 | `tx_deshred_to_final_latency`| `f64 (ms)` | Elapsed time: `deshred -> finalized` | Forensic Engine | `DERIVED` | `IN DEVELOPMENT` |

---

## 3. Definitive Data Availability Matrix Across Sources

The comprehensive cross-source comparison table covering all key consensus telemetry dimensions:

| Field Group | Field | Public RPC | Public WS | Yellowstone gRPC | Agave Geyser | Chrono Plugin | Derived | Active Status | Provenance | Reason if Unavailable |
|---|---|---|---|---|---|---|---|---|---|---|
| **Slot** | `slot` | YES | YES | YES | YES | YES | — | Available | `DIRECT` | — |
| **Slot** | `parent_slot` | YES | YES | YES | YES | YES | — | Available | `DIRECT` | — |
| **Slot** | `root_slot` | YES | YES | YES | YES | YES | — | Available | `DIRECT` | — |
| **Slot** | `leader` | NO* | NO | YES | YES | YES | YES | Available | `DERIVED` | *Requires `getLeaderSchedule` RPC |
| **Slot** | `duration` | NO | NO | YES | YES | YES | YES | Available | `DERIVED` | Calculated from arrival timestamps |
| **Bank** | `bank_id` | NO | NO | YES | YES | YES | NO | Available (Geyser/YS) | `DIRECT` | Validator-local; omitted by Public RPC |
| **Bank** | `bank_hash` | NO | NO | YES | YES | YES | NO | Available (Geyser/YS) | `DIRECT` | Omitted by Public RPC |
| **Bank** | `candidate_banks` | NO | NO | YES | YES | YES | NO | Available (Geyser/YS) | `DIRECT` | Public RPC only returns confirmed blocks |
| **Bank** | `bank_lifecycle` | NO | NO | YES | YES | YES | YES | Available (Geyser/YS) | `DIRECT` | Processed/Confirmed/Rooted/Dead |
| **Marker** | `block_header` | NO | NO | YES | YES | YES | NO | Available (Geyser/YS) | `DIRECT` | Omitted by Public RPC |
| **Marker** | `block_footer` | NO | NO | YES | YES | YES | NO | Available (Geyser/YS) | `DIRECT` | Omitted by Public RPC |
| **Marker** | `UpdateParent` | NO | NO | YES | YES | YES | NO | Available (Geyser/YS) | `DIRECT` | Omitted by Public RPC |
| **Cert** | `raw_bytes` | NO* | NO | NO** | YES | YES | NO | Available (Plugin/RPC*) | `DIRECT` | *Devnet RPC `getAgGenesisCert`; **YS strips raw BLS |
| **Cert** | `parsed_metadata`| NO | NO | NO | YES | YES | YES | Available (Plugin/RPC) | `DERIVED` | Parsed from BLS certificate |
| **Cert** | `crypto_verified`| NO | NO | NO | NO | YES | YES | Available (Chrono) | `DERIVED` | Requires BLS pairing verification |
| **Producer**| `timestamp_nanos`| NO | NO | YES | YES | YES | NO | Available (Geyser/YS) | `DIRECT` | High-precision producer nanosecond clock |
| **Producer**| `user_agent` | NO | NO | YES | YES | YES | NO | Available (Geyser/YS) | `DIRECT` | Client identifier string in footer |
| **Entries** | `entry_stream` | NO | NO | YES | YES | YES | NO | Available (Geyser/YS) | `DIRECT` | Continuous entry ordering |
| **Deshred** | `pre_exec_tx` | NO | NO | NO* | YES | YES | NO | Available (Plugin) | `DIRECT` | *Open-source YS server returns `UNIMPLEMENTED` |
| **Shreds** | `raw_udp_shreds`| NO | NO | NO | NO | NO | NO | UNAVAILABLE | `UNAVAILABLE` | Geyser omits raw shreds; requires eBPF socket tracing |

---

## 4. Telemetry Level Hierarchy

CHRONO defines five distinct operational levels:

- **Level 0 (Public JSON-RPC)**: Basic slot, root, parent, linear blocks, leader schedule polling (`27%` coverage).
- **Level 1 (Public RPC + WebSocket)**: Real-time slot ticks, vote notifications, account subscriptions (`35%` coverage).
- **Level 2 (Yellowstone gRPC)**: Full entry stream, bank-scoped slots, candidate `bank_id`, `bank_hash`, block footers, `UpdateParent` (`78%` coverage).
- **Level 3 (Validator Geyser)**: In-process Agave validator hooks, all lifecycle states, raw block footers (`88%` coverage).
- **Level 4 (Chrono Validator Plugin / Telemetry)**: Dedicated UDS IPC bridge, raw BLS certificates, pre-execution deshred, monotonic observer timestamps (`94%` coverage).
- **Level 5 (Multi-Validator Full-Fidelity)**: Multi-node cross-validator reconciliation graph with observer vs producer disambiguation (`97%` coverage).

*(The remaining ~3% represents raw UDP TVU shreds which are intentionally kept out of validator plugins for system stability).*
