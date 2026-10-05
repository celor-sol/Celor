# CHRONO PHASE 6: GEYSER PLUGIN ARCHITECTURE & SPECIFICATION

> **Document**: `docs/PHASE_6_GEYSER_PLUGIN.md`  
> **Status**: RATIFIED SPECIFICATION & IMPLEMENTATION GUIDE  
> **Crate**: `crates/chrono-geyser-plugin`  
> **Target Library**: `libchrono_geyser_plugin.dylib` (macOS) / `libchrono_geyser_plugin.so` (Linux)

---

## 1. Architectural Mandate & Boundary Separation

Under CHRONO master rules:
1. **Validator Non-Interference**: A Geyser plugin runs directly in the address space of the Solana / Agave validator process. Any blocking I/O, heavy computation, memory allocation sprees, or lock contention in the plugin can poison the validator's replay or banking stage, causing missed leader slots or cluster divergence.
2. **Ingestion Only**: The plugin is strictly an **ingestion bridge**. It NEVER executes consensus resolution, fork selection, or complex graph traversals inside the validator process.
3. **Transport Selection**: High-throughput, non-blocking Unix Domain Socket (UDS) stream (`/tmp/chrono_geyser.sock`) or localhost IPC. Avoid per-event HTTP requests.
4. **Data Normalization**: Translates in-memory Agave Geyser callback structs into strongly-typed, schema-versioned JSONL / binary frames forwarded directly to Chrono Core.

```
┌────────────────────────────────────────────────────────┐
│             AGAVE VALIDATOR PROCESS                    │
│                                                        │
│  BankForks ───► BankingStage ───► Blockstore / Replay  │
│        │               │                 │             │
│        ▼               ▼                 ▼             │
│   update_bank_status notify_deshred notify_block_footer│
│        │               │                 │             │
│  ┌─────┴───────────────┴─────────────────┴──────────┐  │
│  │       CHRONO GEYSER PLUGIN (In-Process)          │  │
│  │  - Non-blocking Ring Buffer (crossbeam/flume)    │  │
│  │  - Zero-copy serializer                          │  │
│  │  - Background UDS writer thread                  │  │
│  └──────────────────────┬───────────────────────────┘  │
└─────────────────────────┼──────────────────────────────┘
                          │ Unix Domain Socket
                          │ (/tmp/chrono_geyser.sock)
                          ▼
┌────────────────────────────────────────────────────────┐
│               CHRONO CORE PROCESS                      │
│                                                        │
│  ValidatorGeyserSource (UDS Reader)                    │
│        │                                               │
│        ▼                                               │
│  Normalizer & Timestamp Anchor                         │
│        │                                               │
│        ▼                                               │
│  EventBus ───► BankGraph / CanonicalResolver           │
│        │                                               │
│        ▼                                               │
│  API Engine (HTTP/WS) ───► UI / CLI / SDK              │
└────────────────────────────────────────────────────────┘
```

---

## 2. Plugin C ABI Entry Point

Agave validator dynamically loads the plugin via `libloading` and resolves `_create_plugin`:

```rust
#[no_mangle]
pub unsafe extern "C" fn _create_plugin() -> *mut dyn GeyserPlugin {
    let plugin = ChronoGeyserPlugin::new();
    let boxed: Box<dyn GeyserPlugin> = Box::new(plugin);
    Box::into_raw(boxed)
}
```

---

## 3. Hook Implementations & Event Translation

### 3.1 Bank-Scoped Slot Lifecycle (`update_bank_status`)
```rust
fn update_bank_status(
    &self,
    slot: Slot,
    parent: Option<u64>,
    status: &SlotStatus,
    bank_id: BankId,
) -> PluginResult<()> {
    self.send_event(RawGeyserPayload::BankStatus {
        slot,
        parent_slot: parent,
        bank_id,
        status: match status {
            SlotStatus::Processed => "Processed",
            SlotStatus::Confirmed => "Confirmed",
            SlotStatus::Rooted => "Rooted",
            SlotStatus::Dead(err) => "Dead",
            SlotStatus::CreatedBank => "Created",
        },
        timestamp_nanos: current_monotonic_nanos(),
    });
    Ok(())
}
```

### 3.2 Alpenglow Block Footer (`notify_block_footer`)
```rust
fn notify_block_footer(
    &self,
    block_footer: ReplicaBlockFooterInfoVersions,
    bank_id: BankId,
) -> PluginResult<()> {
    match block_footer {
        ReplicaBlockFooterInfoVersions::V0_0_1(info) => {
            match info.block_footer {
                VersionedBlockFooter::V1(footer) => {
                    self.send_event(RawGeyserPayload::BlockFooter {
                        slot: info.slot,
                        bank_id,
                        bank_hash: footer.bank_hash.to_string(),
                        producer_time_nanos: footer.block_producer_time_nanos,
                        user_agent: String::from_utf8_lossy(footer.block_user_agent).to_string(),
                        final_cert: footer.block_final_cert.map(|c| CertPayload::from(c)),
                        notar_cert: footer.notar_reward_cert.map(|c| CertPayload::from(c)),
                        skip_cert: footer.skip_reward_cert.map(|c| CertPayload::from(c)),
                        observed_at_nanos: current_monotonic_nanos(),
                    });
                }
            }
        }
    }
    Ok(())
}
```

### 3.3 Fast Leader Handover (`notify_entry_update_parent`)
```rust
fn notify_entry_update_parent(
    &self,
    update_parent: ReplicaEntryUpdateParentInfoVersions,
) -> PluginResult<()> {
    match update_parent {
        ReplicaEntryUpdateParentInfoVersions::V0_0_1(info) => {
            self.send_event(RawGeyserPayload::UpdateParent {
                slot: info.slot,
                cleared_bank_id: Some(info.cleared_bank_id),
                parent_slot: info.parent_slot,
                parent_block_id: Some(info.parent_block_id.to_string()),
                fec_set_index: None,
                source_stream: "entry",
                observed_at_nanos: current_monotonic_nanos(),
            });
        }
    }
    Ok(())
}
```

### 3.4 Deshred Pre-Execution Transactions (`notify_deshred_transaction`)
```rust
fn notify_deshred_transaction(
    &self,
    transaction: ReplicaDeshredTransactionInfoVersions,
    slot: Slot,
) -> PluginResult<()> {
    self.send_event(RawGeyserPayload::DeshredTx {
        slot,
        signature: transaction.signature().to_string(),
        raw_tx_bytes: transaction.raw_bytes().to_vec(),
        observed_at_nanos: current_monotonic_nanos(),
    });
    Ok(())
}
```

---

## 4. Configuration Schema (`chrono-geyser-plugin.json`)

To load the plugin into `solana-test-validator` or a production validator:

```json
{
  "libpath": "/Users/prakhargaur/Desktop/CHRONO/target/debug/libchrono_geyser_plugin.dylib",
  "uds_socket_path": "/tmp/chrono_geyser.sock",
  "observer_validator_id": "LocalValidatorNode1",
  "enable_bank_status": true,
  "enable_block_footer": true,
  "enable_update_parent": true,
  "enable_deshred": true,
  "enable_entries": true,
  "channel_buffer_capacity": 65536
}
```

---

## 5. Backpressure, Concurrency & Security

1. **Non-blocking Dispatch**:
   The callback handlers push into a bounded multi-producer single-consumer (MPSC) ring buffer (capacity: 65,536 events). If the buffer fills because Chrono Core is disconnected or slow, older non-critical events are dropped and counter `events_dropped_total` is incremented.
2. **Critical Event Invariant**:
   `UpdateParent`, `BankStatus::Rooted`, and `BlockFooter` events are flagged high-priority and prioritized in the ring buffer.
3. **Security Invariant**:
   The plugin NEVER accesses `validator_identity.json`, vote keys, or gossip private state. It only consumes public read-only references passed across the Geyser trait boundary.
