# CHRONO Service Normalized Event Schema (v1)

> **Schema Version**: `1`  
> **Target Format**: JSON / Serde  
> **Source Representation**: `ChronoServiceEvent` in `crates/chrono-server/src/envelope.rs`

---

## 1. Top-Level Event Envelope

Every event dispatched over the WebSocket `/api/v1/stream` or stored in the bounded snapshot history is wrapped in the canonical `ChronoServiceEvent` envelope:

```json
{
  "schema_version": 1,
  "sequence": 1042,
  "event_id": 1042,
  "cluster": "testnet",
  "source": "rpc-ws",
  "environment": "live",
  "observed_at_ms": 1791000000100,
  "received_at_ms": 1791000000102,
  "slot": 341829100,
  "bank_id": null,
  "blockhash": "8xMvY3ZqW9...8QfA",
  "parent_slot": 341829099,
  "parent_blockhash": "7wLvX2YpT8...7PeZ",
  "provenance": "DIRECT",
  "event_type": "CanonicalChanged",
  "payload": {}
}
```

### Field Definitions:

| Field | Type | Description |
|-------|------|-------------|
| `schema_version` | `u32` | Schema major version. Current is `1`. |
| `sequence` | `u64` | Monotonically increasing service-local sequence ID. Used for gap detection and ring buffer replay. |
| `event_id` | `u64` | Globally unique event identifier within the session. |
| `cluster` | `string` | Target cluster identifier (`testnet`, `devnet`, `mainnet-beta`, `local-geyser`). |
| `source` | `string` | Ingest source driver (`rpc-ws`, `yellowstone-grpc`, `local-geyser-fixture`). |
| `environment` | `string` | Execution environment (`live`, `local`, `fixture`). |
| `observed_at_ms` | `u64` | Monotonic timestamp when the physical event was emitted by the validator/network. |
| `received_at_ms` | `u64` | Monotonic timestamp when Chrono Core ingested the event. |
| `slot` | `u64` | Solana slot associated with this event. |
| `bank_id` | `u64?` | Validator-local candidate bank index (populated if Geyser/Yellowstone provides it, `null` if public RPC). |
| `blockhash` | `string?` | Canonical blockhash associated with this bank/block. |
| `parent_slot` | `u64?` | Slot of the parent bank/block. |
| `parent_blockhash`| `string?` | Blockhash of the parent bank/block. |
| `provenance` | `string` | Provenance classification: `DIRECT`, `DERIVED`, `ESTIMATED`, `UNAVAILABLE`. |
| `event_type` | `string` | Exact event variant name. |
| `payload` | `object` | Event-specific structured data. |

---

## 2. Event Types & Payloads

### 2.1 `SlotProgress`
Emitted as the 200–250ms slot window elapses.

```json
{
  "event_type": "SlotProgress",
  "payload": {
    "slot": 341829100,
    "elapsed_ms": 125,
    "target_duration_ms": 250,
    "phase_ratio": 0.50
  }
}
```

### 2.2 `LeaderTransition`
Emitted when a leader slot boundary is reached or handoff scheduled.

```json
{
  "event_type": "LeaderTransition",
  "payload": {
    "current_leader": "7Np41oeYqPefeNQEHSv1UDhYrehxin3NStELs8ZAKTUJ",
    "next_leader": "Gsf5P4QxW8n1YgY...9aB",
    "handoff_state": "ACTIVE"
  }
}
```

### 2.3 `CandidateBankCreated`
Emitted when a candidate bank branch is detected for a slot.

```json
{
  "event_type": "CandidateBankCreated",
  "bank_id": 1,
  "blockhash": "8xMvY3ZqW9...8QfA",
  "payload": {
    "bank_id": "bank-1",
    "raw_bank_id": 1,
    "slot": 341829100,
    "parent_bank_id": "bank-0",
    "tx_count": 840,
    "state": "OBSERVED"
  }
}
```

### 2.4 `CanonicalChanged`
Emitted when the canonical resolver determines the winning branch for a slot.

```json
{
  "event_type": "CanonicalChanged",
  "bank_id": 1,
  "blockhash": "8xMvY3ZqW9...8QfA",
  "payload": {
    "slot": 341829100,
    "canonical_bank_id": "bank-1",
    "reason": "Longest valid notarized lineage"
  }
}
```

### 2.5 `UpdateParent`
Emitted during fast leader handover when a leader discards an unfinalized predecessor bank and binds to an earlier ancestor.

```json
{
  "event_type": "UpdateParent",
  "payload": {
    "slot": 341829100,
    "cleared_bank_id": 1,
    "replacement_bank_id": 2,
    "parent_slot": 341829098,
    "parent_block_id": "bank-2",
    "reason": "Fast leader handover UpdateParent"
  }
}
```

### 2.6 `FinalityChanged`
Emitted upon consensus notarization / finalization.

```json
{
  "event_type": "FinalityChanged",
  "payload": {
    "slot": 341829099,
    "cert_type": "FinalCert",
    "latency_ms": 114,
    "stake_percent": 80.0
  }
}
```

### 2.7 `BlockFooterObserved`
Emitted upon block execution completion with Alpenglow timing and certificates.

```json
{
  "event_type": "BlockFooterObserved",
  "bank_id": 1,
  "payload": {
    "slot": 341829100,
    "bank_id": 1,
    "bank_hash": "4vJ9JU1bJJE96nDqasdf8899aabbccddeeff",
    "producer_time_nanos": 1791000000100420000,
    "user_agent": "agave/v2.0.1",
    "has_final_cert": true,
    "has_notar_cert": false
  }
}
```

### 2.8 `CertificateObserved`
Emitted when a raw BLS consensus certificate is observed on the validator stream.

```json
{
  "event_type": "CertificateObserved",
  "payload": {
    "slot": 341829100,
    "kind": "FinalCert",
    "raw_len": 192,
    "has_raw_bytes": true,
    "block_id": "blk-341829100",
    "validation_status": "PARSED (UNVERIFIED)",
    "stake_percent": 80.0
  }
}
```

### 2.9 `DeshredObserved`
Emitted when a transaction is reconstructed from shreds prior to execution.

```json
{
  "event_type": "DeshredObserved",
  "payload": {
    "slot": 341829100,
    "signature": "5wK9JU1bJJE96nDq...99aabbcc",
    "raw_tx_len": 240,
    "pre_execution_timestamp_nanos": 1791000000050000000,
    "static_accounts": ["11111111111111111111111111111111", "ComputeBudget111111111111111111111111111111"],
    "fec_set_index": 0
  }
}
```

### 2.10 `EntryObserved`
Emitted when an entry batch is produced or received.

```json
{
  "event_type": "EntryObserved",
  "payload": {
    "slot": 341829100,
    "bank_id": 1,
    "entry_index": 4,
    "tx_count": 64
  }
}
```

### 2.11 `ProducerTimingObserved`
Emitted with nanosecond-precision producer timing forensics.

```json
{
  "event_type": "ProducerTimingObserved",
  "payload": {
    "slot": 341829100,
    "producer_time_nanos": 1791000000100420000,
    "chrono_received_at_nanos": 1791000000102600000,
    "interval_nanos": 2180000,
    "interval_ms": 2.18
  }
}
```

### 2.12 `BankStatusChanged`
Emitted when a candidate bank transitions through lifecycle stages (`CreatedBank`, `Processed`, `Confirmed`, `Rooted`, `Dead`).

```json
{
  "event_type": "BankStatusChanged",
  "bank_id": 1,
  "payload": {
    "bank_id": 1,
    "status": "Confirmed"
  }
}
```

---

## 3. Disambiguation: Observer vs Producer

Every telemetry record supports the `observer` context:

```json
{
  "observer": {
    "observing_validator": "Local Agave 2.0.1 (Validator A)",
    "producing_validator": "Validator B",
    "transport_type": "uds",
    "cluster_environment": "local-validator"
  }
}
```

**Invariant**: `bank_id` is strictly local to `observing_validator`. `producer_time_nanos` is emitted by `producing_validator`.
Cross-validator reconciliation must use deterministic `(slot, blockhash)`, never `bank_id`.

