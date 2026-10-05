# CHRONO Phase 7 — Multi-Provider Data Plane Matrix

> **Document Status**: Production Complete  
> **Target Engine**: CHRONO Ingestion & Evidence Graph  
> **Supported Inputs**: Public RPC, WebSocket, Yellowstone gRPC, Local Geyser IPC, CHRONO Geyser Plugin

---

## 1. Supported Input Providers

CHRONO is completely provider-independent. Core business logic and internal state models have zero coupling to commercial vendor APIs.

| Provider Identifier | Transport | Semantics | Authority Scope | Typical Latency |
| :--- | :--- | :--- | :--- | :--- |
| `public_rpc` | HTTP JSON-RPC | Polled blockhash, slot, and commitment status. | Low | 400ms – 1200ms |
| `rpc_websocket` | WebSocket | Streaming slot, block, and program account updates. | Medium | 150ms – 400ms |
| `yellowstone_grpc` | HTTP/2 gRPC | High-throughput protobuf stream (transactions, blocks, slots). | High | 50ms – 150ms |
| `local_geyser_uds` | Unix Domain Socket | High-speed IPC stream direct from local Agave validator node. | Highest (Local Node) | 0.5ms – 2ms |
| `chrono_geyser_plugin` | C ABI / IPC | Zero-copy plugin inside Agave validator address space. | Highest (Validator Local) | <0.1ms (in-process) |

---

## 2. Multi-Layer Normalization Pipeline

Raw data is transformed through 5 distinct, immutable layers. Lower layers are never destroyed:

```
[ RAW PROVIDER STREAM ]
         ↓
[ 1. RAW LAYER ] (Exact byte payload, provider ID, arrival timestamp)
         ↓
[ 2. NORMALIZED LAYER ] (ChronoEvent: standardized types, field-level provenance)
         ↓
[ 3. RECONCILED LAYER ] (SourceEvidenceGraph: cross-provider deduplication & conflicts)
         ↓
[ 4. CANONICAL LAYER ] (CoreStateEngine: consensus-verified canonical fork & finality)
         ↓
[ 5. HIGH-LEVEL LAYER ] (ApplicationEvent: typed, developer-ready events)
```

---

## 3. Deduplication Architecture

### 3.1 Event Identity Standardization
- Multi-provider streams inevitably deliver the same real-world event multiple times.
- Cross-provider event identity is computed from `(slot, blockhash, event_semantic_hash)`.
- Local validator identifiers such as `bank_id` are strictly isolated from global event identity.

### 3.2 Deduplication Invariant
- Identical observations from the same provider or across providers update provenance and confirmation weight, but **never duplicate state entries or trigger duplicate downstream application events**.
- Empirically validated in `test_phase7_multi_provider_reconciliation_and_deduplication`: 10,000 duplicate observations of the same blockhash produced exactly 1 recorded observation entry.

---

## 4. Conflict Resolution & Authority Ranking

When multiple providers report contradictory data for the same slot, the `SourceEvidenceGraph` classifies the conflict:

| Status | Definition | Resolution Rule |
| :--- | :--- | :--- |
| `Match` | Multiple providers report identical blockhashes and state. | State confirmed across multiple vantage points. |
| `Conflict` | Providers report differing blockhashes or parent lineages for slot $S$. | Quarantine contradictory branches; inspect protocol evidence. |
| `Unresolved` | Insufficient evidence to establish canonical branch. | Both branches remain `Candidate`; execution engine pauses. |
| `Resolved` | Cryptographic certificate or sealed block footer confirms winning branch. | Winning fork promoted to `Canonical`; superseded branch marked `Abandoned`. |

### Authority Precedence Hierarchy
1. **Local Geyser / Direct IPC**: Highest authority for validator-local candidate bank lifecycle and UpdateParent markers.
2. **Yellowstone gRPC / Raw Shreds**: Authoritative for fast block streaming, entries, and transaction arrival times.
3. **Public RPC / WebSocket**: Secondary confirmation and fallback validation.

---

## 5. Failover & Connection Health Model

### 5.1 Health States
Every registered provider stream maintains an observable health state:
- `connected`: Active stream receiving heartbeats and valid frames.
- `catching_up`: Reconnected stream syncing recent historical slots.
- `healthy`: Event latency and slot lag within cluster tolerances (<2 slots).
- `degraded`: Event arrival latency exceeding threshold or partial packet drop.
- `lagging`: Slot lag $\ge 4$ slots behind highest observed slot.
- `stale`: No events received within timeout window ($>3000\text{ms}$).
- `disconnected`: Network socket severed; reconnection backoff active.

### 5.2 Seamless Failover
If the primary provider stream severs:
1. Active state machine continues uninterrupted using secondary/tertiary streams.
2. The execution engine flags `source_tier: Degraded` if single-source failover reduces redundancy below threshold.
3. Upon primary stream reconnection, historical overlap is deduplicated automatically. Sequence gaps are flagged (`DATA_GAP`) without resetting the canonical state tree.

---

## 6. Backpressure & Memory Bounds

- **Ring Buffers**: Event queues use fixed-capacity pre-allocated ring buffers (default 65,536 events).
- **Drop Policy**: Non-critical telemetry events (e.g. detailed account poll ticks) are dropped with explicit counters (`dropped_events_count`) when capacity limits are hit.
- **Critical Consensus Guarantee**: Critical consensus events (`BlockFooterObserved`, `CertificateObserved`, `UpdateParent`) have dedicated priority queues and are **never silently dropped**.
- **Memory Footprint**: Memory usage remains strictly bounded (<100MB under 50,000 events/sec sustained burst).
