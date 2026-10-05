# CHRONO Architecture Decision Records (ADRs)

---

### ADR-001: Strict Provider Independence
- **Date**: 2026-10-03
- **Status**: `ACCEPTED`
- **Decision**: CHRONO will not couple its core data structures, event models, or API boundaries to any specific third-party provider (such as Helius, Triton, Jito, or QuickNode). All external streaming ingestion must flow through swappable Provider Adapters.
- **Reason**: Vendor lock-in exposes infrastructure to proprietary pricing, breaking schema changes, and single-point-of-failure vulnerabilities. CHRONO must remain a universal open layer operable across any validator, public RPC, or private Geyser stream.

---

### ADR-002: $0-First Constraint (Free-Tier & Devnet/Testnet Bootstrap)
- **Date**: 2026-10-03
- **Status**: `ACCEPTED`
- **Decision**: Bootstrap, testing, and Phase 1 verification will run exclusively on Solana Devnet/Testnet and free-tier infrastructure (open-source tools, public RPCs, free Yellowstone endpoints, local test validator).
- **Reason**: Ensures accessibility, avoids premature capital expenditure before protocol stability, and proves that performance advantages stem from architectural superiority rather than expensive hardware subsidies.

---

### ADR-003: Devnet/Testnet First Before Mainnet
- **Date**: 2026-10-03
- **Status**: `ACCEPTED`
- **Decision**: All consensus verification, bank graph tracking, and `UpdateParent` testing must be proven on Devnet/Testnet clusters before attempting any Mainnet deployments.
- **Reason**: Alpenglow features (Agave 4.3, Votor, multi-bank slots, `UpdateParent`) are actively rolling out on Devnet/Testnet. Devnet provides a safe, active environment to test bank invalidation without financial risk.

---

### ADR-004: Explicit Prohibition of Rotor Assumptions
- **Date**: 2026-10-03
- **Status**: `ACCEPTED`
- **Decision**: CHRONO will NOT assume Rotor block propagation is live or operational. The system will assume standard Turbine block propagation while maintaining an extensible interface for future Rotor integration.
- **Reason**: While SIMD-0326 describes Rotor, Rotor has been explicitly deferred from the initial Agave Votor rollouts. Assuming Rotor is active in live environments would cause critical timing and networking failures.

---

### ADR-005: Preservation of Legacy Control Paths for Empirical A/B Benchmarking
- **Date**: 2026-10-03
- **Status**: `ACCEPTED`
- **Decision**: The legacy single-bank, 400ms-assumption polling/WebSocket processing paths must be retained alongside the new Alpenglow-aware bank graph engine.
- **Reason**: Demonstrating speed and reliability advantages requires concurrent, empirical A/B testing on the exact same stream. Removing legacy paths makes scientific performance comparison impossible.

---

### ADR-006: Empirical Measurement Before Optimization
- **Date**: 2026-10-03
- **Status**: `ACCEPTED`
- **Decision**: No micro-optimization or speculative architectural complexity may be introduced without prior baseline measurement proving a tangible bottleneck.
- **Reason**: Premature optimization introduces technical debt and fragility. All optimization must be justified by profiling data adhering to `BENCHMARK_PROTOCOL.md`.

---

### ADR-007: Explicit Canonical Resolution (Never Assume Observed == Canonical)
- **Date**: 2026-10-03
- **Status**: `ACCEPTED`
- **Decision**: Ingested blocks and entries must remain in a `Candidate` state until explicitly confirmed canonical by block sealing, blockhash inclusion, or finality certificate verification.
- **Reason**: Fast leader handover produces multiple candidate banks per slot. Treating an observed bank as canonical before resolution leads to executing trades on abandoned forks.

---

### ADR-008: Cross-Provider Reconciliation via Blockhash & Block Identity (Not `bank_id`)
- **Date**: 2026-10-03
- **Status**: `ACCEPTED`
- **Decision**: Cross-provider deduplication and reconciliation must key on `(slot, blockhash)` and parent linkages, NEVER on `bank_id`.
- **Reason**: `bank_id` is validator-local in Agave 4.3. Two different providers observing the exact same bank assign different internal `bank_id` values. Relying on `bank_id` across providers creates false fork splits and reconciler corruption.

---

### ADR-009: In-Memory Consensus Pipeline Architecture & Measured Latency Baseline
- **Date**: 2026-10-03
- **Status**: `ACCEPTED`
- **Decision**: The Chrono Core ingestion and consensus pipeline is built as a zero-disk, zero-DB in-memory actor/bus pipeline in Rust. State transitions between raw observation, normalizer, BankGraph, canonical resolution, and finality tracking are bounded to microsecond latencies.
- **Reason**: Latency measurement under release profiling over N=10,000 iterations recorded an end-to-end median pipeline overhead of 2.0µs (p50: 2,000ns, p95: 3,834ns, p99: 8,583ns [MEASURED]). This provides sufficient overhead headroom for future 200ms slot execution without external DB latency contamination.

---

### ADR-010: Zero-Fabrication of Unsupported Public RPC Capabilities
- **Date**: 2026-10-03
- **Status**: `ACCEPTED`
- **Decision**: When running against free-tier public Solana RPC/WebSocket endpoints that omit internal validator telemetry (such as local `bank_id` or streaming `UpdateParent` events), CHRONO must explicitly mark capabilities as `UNSUPPORTED` and fields as `UNKNOWN`.
- **Reason**: Trust and protocol integrity are foundational invariants. Fabricating simulated multi-bank candidate splits or fake `UpdateParent` notifications on live cluster streams destroys analytical validity.

---

### ADR-011: Historical Retention of Abandoned Candidate Banks (No-Deletion Policy)
- **Date**: 2026-10-03
- **Status**: `ACCEPTED`
- **Decision**: When an `UpdateParent` handover or fork resolution invalidates a candidate bank, the BankGraph marks its state as `BankState::Abandoned` along with the specific abandonment reason, but NEVER deletes the node from memory (subject to bounded slot pruning).
- **Reason**: Forensics, post-mortem trade failure autopsy, and latency analysis require understanding why a candidate bank was rejected rather than having it vanish without historical trace.

---

### ADR-012: Adoption of Optimus Design System for Technical Editorial UI
- **Date**: 2026-10-03
- **Status**: `ACCEPTED`
- **Decision**: The Chrono user interface adopts the high-precision, technical editorial aesthetic of the Optimus design system (OKLCH light base, Instrument Serif & Sans typography, JetBrains Mono telemetry, hairline borders, noise overlay, and kinetic ASCII glyph canvas) while completely replacing all branding and product copy with CHRONO protocol infrastructure.
- **Reason**: Avoids generic neon crypto terminal tropes or cluttered block explorer aesthetics. Establishes a calm, austere, high-trust developer and operator presentation.

---

### ADR-013: UI Data Adapter Decoupling & Multi-Route Architecture
- **Date**: 2026-10-03
- **Status**: `ACCEPTED`
- **Decision**: The UI strictly consumes Chrono Core consensus models via a typed UI Data Adapter layer (`ChronoService` and `ChronoEventBus`). React components contain zero protocol business logic, timing calculations, or fork choice heuristics. The application is cleanly partitioned into four dedicated App Router views: Live (`/`), Transaction Autopsy (`/transaction`), Network & Capabilities (`/network`), and Developer Studio (`/developers`).
- **Reason**: Prevents protocol logic duplication, eliminates React re-render performance bottlenecks, and guarantees strict source transparency across public RPC and high-speed Geyser streams.

---

### ADR-014: Full Alpenglow Telemetry & Four-Tier Provenance Tracking
- **Date**: 2026-10-03
- **Status**: `ACCEPTED`
- **Decision**: Every single Alpenglow field exposed by Chrono Core must be tagged with explicit provenance: `DIRECT`, `DERIVED`, `ESTIMATED`, or `UNAVAILABLE`.
  - `bank_id`: Scoped strictly to `(slot, bank_id)` within a single provider, NEVER equated across providers.
  - Cross-provider reconciliation standard: `(slot, blockhash)` matching.
  - `bank_hash`: Handled as an account state delta accumulator, strictly distinct from `blockhash`.
  - Alpenglow block footer: Captures `block_producer_time_nanos`, `block_user_agent`, `block_final_cert`, `skip_reward_cert`, and `notar_reward_cert`.
- **Reason**: Protocol transparency is paramount. The system must never spoof or fabricate missing fields on limited feeds, while providing rich high-fidelity telemetry when connected to Yellowstone gRPC or Local Geyser plugins.

---

### ADR-015: Local Full-Fidelity Mode ($0 Budget Invariance)
- **Date**: 2026-10-03
- **Status**: `ACCEPTED`
- **Decision**: Chrono provides a dedicated Local Full-Fidelity Mode (`LocalGeyserModeAdapter` and `chrono inspect --local`) capable of emitting and validating 100% of Alpenglow protocol fields without incurring cloud or commercial RPC expenses.
- **Reason**: Allows researchers and developers to test candidate bank succession, fast leader handover `UpdateParent` events, replacement bank correlations, and BLS certificate verifications with $0 infrastructure spend.

---

### ADR-016: Strict Separation of Rust Consensus Authority and Frontend Presentation Adapter
- **Date**: 2026-10-03
- **Status**: `ACCEPTED`

---

### ADR-017: Funding Preflight, Safe Signer Hierarchy, and Live Interleaved Benchmark Validation
- **Date**: 2026-10-04
- **Status**: `ACCEPTED`
- **Decision**: All live cluster execution benchmarks in Chrono Phase 4.1 must enforce mandatory funding preflight checks prior to trial 0 (`chrono bench preflight`). The runner verifies balance sufficiency for the planned sample budget and safely halts with `INSUFFICIENT_FUNDS` if balance is inadequate, strictly avoiding unfunded false-negative execution runs. The historical unfunded run (`20261003-185228-testnet-rpc`) is formally classified as `INVALID_UNFUNDED_RUN` and excluded from performance claims. Live experiments enforce strictly interleaved A/B assignment ($C \to H \to C \to H$), self-transfers to eliminate rent-exemption creation overhead, and independent on-chain confirmation verification.
- **Reason**: Guarantees experimental integrity, prevents secret key exposure, provides reproducible empirical baselines, and ensures zero unscientific claims under real Solana cluster conditions.

---

### ADR-018: Direct Leader TPU Transport via Agave 4.3 IETF QUIC
- **Date**: 2026-10-04
- **Status**: `ACCEPTED`
- **Decision**: In Chrono Phase 5, all direct validator execution pathways must use standard IETF QUIC over UDP (`solana-quic-client` and `QuicConnectionCache`) targeting validator `tpuQuic` ports (typically 8003). CHRONO strictly rejects any raw UDP TPU packet implementation because Agave 4.3 has completely deprecated and disabled raw UDP TPU ingestion (`tpu: null` across 100% of cluster nodes). Node addresses are resolved through `LeaderTransportResolver` with a 60-second TTL cache (< 500ns lookup). Connection prewarming and dual-targeting (`CurrentPlusNext`) during slot-tail windows eliminate cold handshake penalties.
- **Reason**: Aligns with official Agave 4.3 networking invariants, prevents immediate firewall packet dropping, mitigates cold TLS 1.3 handshake latency, and provides zero-hop leader delivery without commercial vendor lock-in.

---

### ADR-019: 2x2 Factorial Benchmark Matrix for Transport & Consensus Disambiguation
- **Date**: 2026-10-04
- **Status**: `ACCEPTED`
- **Decision**: To definitively isolate the performance effects of consensus timing from transport network hops, CHRONO implements a 2x2 Factorial experimental design:
  - Route A: Control + RPC (Legacy Baseline)
  - Route B: Chrono-Aware + RPC (Phase 4.1 Pipeline)
  - Route C: Control + QUIC (Direct Transport Isolation)
  - Route D: Chrono-Aware + QUIC (Full Chrono Execution System)
  Trials are executed in strict 4-way interleaved round-robin order ($A \to B \to C \to D$). All conclusions are categorized strictly under empirical verdicts (`CONFIRMED ADVANTAGE`, `CONDITIONAL ADVANTAGE`, `NO MATERIAL ADVANTAGE`, or `INCONCLUSIVE`) with statistical significance reporting.
- **Reason**: Prevents confounding transport network queueing with consensus timing logic, ensures temporal variance neutrality, and adheres to strict scientific honesty.
