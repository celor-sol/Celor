# CHRONO PHASE 6: DESHRED & PRE-EXECUTION TELEMETRY ARCHITECTURE

> **Document**: `docs/PHASE_6_DESHRED.md`  
> **Status**: APPROVED & PROTOCOL-ALIGNED  
> **Subject**: Pre-execution transaction observability, Agave deshred hooks, Yellowstone server reality, and pipeline latency forensics.

---

## 1. What is Deshredding?

In Solana's pipeline, transactions transmitted over Turbine are broken down into **shreds** (~1,228 bytes each). When a receiving validator captures shreds from the network:
1. `WindowService` / `Blockstore` collects data and coding (FEC) shreds.
2. Reed-Solomon erasure decoding reconstructs the underlying serialized byte stream.
3. Deshredding parses transactions out of that reconstructed stream **BEFORE** the BankingStage executes them or verifies account balances.

Observing transactions at the deshred layer grants high-frequency systems a **pre-execution latency lead** of 15ms to 80ms over standard post-execution transaction notifications.

---

## 2. Ingestion Pathways & Source Reality Audit

| Pathway | Mechanism | Availability | Status in Phase 6 |
|---|---|---|---|
| **Agave Geyser Hook** | `notify_deshred_transaction` | `IN DEVELOPMENT` (Agave 4.3+) | **DIRECTLY SUPPORTED** via Chrono Geyser Plugin |
| **Yellowstone Protobuf** | `SubscribeDeshred` | Stub in `geyser.proto` | Supported in client schema |
| **Open-Source Yellowstone Server**| `yellowstone-grpc-geyser` | **NOT IMPLEMENTED** | Returns `Status::unimplemented()` |
| **Triton Private Extension** | Proprietary validator fork | Commercial / Paid | Closed-source; vendor-locked |
| **Public RPC / WS** | Standard JSON-RPC | **UNAVAILABLE** | RPC only emits post-execution blocks |

### Critical Finding on Open-Source Yellowstone
Many third-party tools mistakenly advertise Yellowstone deshredding because the RPC method exists in `geyser.proto`. However, inspecting the open-source codebase of `rpcpool/yellowstone-grpc` reveals:
```rust
async fn subscribe_deshred(
    &self,
    _request: Request<SubscribeDeshredRequest>,
) -> Result<Response<Self::SubscribeDeshredStream>, Status> {
    Err(Status::unimplemented("SubscribeDeshred is not implemented in open-source server"))
}
```
**Chrono Mandate**: Chrono does NOT mark deshred "available" on Yellowstone unless connected to an authenticated extension endpoint that actually streams frames. On open-source / local stacks, Chrono relies directly on its in-process validator plugin hook `notify_deshred_transaction`.

---

## 3. Deshred Hook Signature & Semantics

In `agave-geyser-plugin-interface`:
```rust
fn notify_deshred_transaction(
    &self,
    transaction: ReplicaDeshredTransactionInfoVersions,
    slot: Slot,
) -> Result<()>
```

### Pre-Execution Data Payload:
- `slot`: Target execution slot.
- `signature`: Primary 64-byte Ed25519 transaction signature.
- `raw_transaction`: Serialized transaction bytes.
- `static_account_keys`: Array of declared public keys.
- `recent_blockhash`: Blockhash used by the client.
- **OPTIONAL**: `loaded_addresses` (populated only if `deshred_transaction_alt_resolution_enabled()` returns true).

### What Deshred Does NOT Provide:
- `execution_error`: Transaction has not run yet; it may succeed, fail, or be dropped.
- `compute_units_consumed`: Unknown until VM execution.
- `post_balances` / `inner_instructions`: Generated only post-execution.

---

## 4. Entry UpdateParent vs. Deshred UpdateParent

In fast leader handoff (SIMD-0337), `UpdateParent` markers can be received on two distinct internal streams:
1. **`notify_entry_update_parent`**: Fired when an `UpdateParent` marker is decoded from sealed ledger entries.
2. **`notify_deshred_update_parent`**: Fired at the FEC-set boundary in the deshred stream *ahead* of completed entry assembly.

Chrono preserves both markers independently:
- `deshred_update_parent` signals an immediate speculatory purge of the candidate bank queue.
- `entry_update_parent` confirms ledger-level boundary redirection.

---

## 5. End-to-End Latency Forensics Timeline

When validator-side deshred telemetry is active, Chrono measures the micro-timing breakdown:

$$T_{\text{deshred}} \xrightarrow{\Delta t_1} T_{\text{bank\_ingress}} \xrightarrow{\Delta t_2} T_{\text{executed}} \xrightarrow{\Delta t_3} T_{\text{confirmed}} \xrightarrow{\Delta t_4} T_{\text{finalized}}$$

- $\Delta t_1 = T_{\text{bank\_ingress}} - T_{\text{deshred}}$: Reassembly to BankingStage queue flight.
- $\Delta t_2 = T_{\text{executed}} - T_{\text{bank\_ingress}}$: Transaction execution duration.
- $\Delta t_3 = T_{\text{confirmed}} - T_{\text{executed}}$: Cluster voting & supermajority aggregation latency.
- $\Delta t_4 = T_{\text{finalized}} - T_{\text{confirmed}}$: Finality certificate / root settlement latency.
