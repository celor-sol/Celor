# CHRONO Service Architecture & State Machine

> **Architecture Status**: Implemented & Verified in `crates/chrono-server`  
> **Host Model**: Local-first daemon (`127.0.0.1:8900`)  
> **Source Model**: Abstract, swappable provider ingest (`ChronoSource`)  
> **State Engine**: Authoritative, in-memory, thread-safe `CoreStateEngine`

---

## 1. High-Level System Architecture

```
           SOLANA CLUSTER (Testnet / Devnet / Local / Geyser)
                                  │
                                  ▼
               ┌──────────────────────────────────────┐
               │    Abstract ChronoSource Driver      │
               │  - RpcWsSource (Free tier $0)        │
               │  - YellowstoneSource (gRPC)          │
               │  - LocalGeyserFixtureSource (Sandbox)│
               └──────────────────┬───────────────────┘
                                  │
                                  ▼
               ┌──────────────────────────────────────┐
               │         CoreStateEngine              │
               │  - SlotClock (250ms SIMD-0525)       │
               │  - LeaderEngine                      │
               │  - BankGraph                         │
               │  - CanonicalResolver                 │
               │  - FinalityEngine                    │
               │  - CertificateEngine                 │
               └──────────────────┬───────────────────┘
                                  │
                ┌─────────────────┴─────────────────┐
                ▼                                   ▼
   ┌─────────────────────────┐         ┌─────────────────────────┐
   │ Monotonic Sequencer     │         │ Ring Buffer History     │
   │ (Seq #1, #2, #3...)     │         │ (Capacity = 2,048)      │
   └────────────┬────────────┘         └────────────┬────────────┘
                │                                   │
                ▼                                   ▼
   ┌─────────────────────────────────────────────────────────────┐
   │                  Axum HTTP & WebSocket API                  │
   │           GET /api/v1/snapshot | WS /api/v1/stream          │
   └──────────────────────────────┬──────────────────────────────┘
                                  │
                                  ▼
   ┌─────────────────────────────────────────────────────────────┐
   │                  TypeScript ChronoClient                    │
   │  - Zero direct Solana RPC/WS network calls in browser       │
   │  - Auto-reconnect with exponential backoff & gap recovery   │
   │  - Presentation layer for React UI                          │
   └─────────────────────────────────────────────────────────────┘
```

---

## 2. Ingest Source Abstraction (`ChronoSource`)

The `ChronoSource` trait decouples Chrono Core from any specific RPC or streaming vendor:

```rust
#[async_trait]
pub trait ChronoSource: Send + Sync {
    async fn subscribe(&self) -> Result<mpsc::Receiver<ChronoServiceEvent>, Box<dyn std::error::Error + Send + Sync>>;
    fn cluster_name(&self) -> &str;
    fn source_name(&self) -> &str;
    fn environment(&self) -> &str;
    fn rpc_endpoint(&self) -> &str;
}
```

Implementations:
1. `RpcWsSource`: Connects to standard Solana public WebSockets (`slotSubscribe`, `rootSubscribe`) and JSON-RPC (`getSlot`, `getLeaderSchedule`). Operates under the $0 budget constraint.
2. `YellowstoneSource`: Consumes Yellowstone gRPC streams with sub-shred and transaction level visibility.
3. `LocalGeyserFixtureSource`: A deterministic Alpenglow test harness generating candidate banks, `UpdateParent` events, and BLS finality certificates.

---

## 3. Core State Machine & Event Pipeline

The `CoreStateEngine` maintains the single authoritative state representation:

```rust
pub struct CoreStateEngine {
    state: RwLock<EngineInner>,
    sequence_allocator: AtomicU64,
    event_buffer: RwLock<VecDeque<ChronoServiceEvent>>,
    buffer_capacity: usize,
    event_sender: broadcast::Sender<ChronoServiceEvent>,
}
```

### Invariant Rules:
1. **Strict Monotonic Sequencing**: Every ingested event is assigned a sequence number via atomic increment:
   ```rust
   let seq = self.sequence_allocator.fetch_add(1, Ordering::SeqCst);
   ```
2. **Deterministic Bank Tree Updates**:
   - Ingesting a `Bank` event calls `BankGraph::add_bank(...)`.
   - Ingesting an `UpdateParent` marks candidate banks as `ABANDONED` and triggers immediate state pruning.
   - Calling `CanonicalResolver::resolve(...)` identifies the winning tip of the tree.
3. **Ring Buffer Replay**: The server stores the last `N = 2,048` events. When a client reconnects and provides `last_sequence`, events from `last_sequence + 1` up to current are served instantaneously from RAM without requiring a full snapshot transfer.
4. **Slow-Consumer Backpressure**: WebSocket connections use bounded tokio channels. If a client lags and drops below delivery thresholds, the connection is closed to protect daemon memory.
