# CHRONO Phase 1 Implementation Workflow

> **Purpose**: A strictly ordered, repeatable workflow to guide future module-by-module implementation of CHRONO Phase 1.  
> **Status**: Ready for execution when authorized. Do NOT execute automatically during bootstrap.

---

## Step 1: Read Project Context
- Review `docs/CHRONO_CONTEXT.md` to confirm the current module's role within project boundaries.
- Re-verify adherence to the **$0 budget constraint** and provider-independence invariants.

## Step 2: Read Alpenglow & Protocol Specification
- Read `docs/ALPENGLOW_SPEC.md` for verified protocol constants, state models, and callback semantics relevant to the target module.
- Check `docs/OPEN_QUESTIONS.md` to ensure the module does not rely on unresolved protocol ambiguities.

## Step 3: Inspect Target Cluster Environment
- Connect to target test environment (Localnet `solana-test-validator`, Devnet, or Testnet).
- Verify cluster node version, slot progression, and active feature gates via `getFeatureStatus` RPC.

## Step 4: Verify Available Capabilities
- Verify whether the cluster provides:
  - Standard JSON-RPC / WebSocket subscriptions (`slotSubscribe`, `blockSubscribe`).
  - Yellowstone gRPC stream with `bank_id` metadata.
  - Active SIMD-0525 slot duration (300ms, 250ms, or 200ms).
- Ensure required local ports or free-tier credentials are functional without cost.

## Step 5: Implement One Isolated Module
- Implement strictly one component according to `docs/ARCHITECTURE.md` (e.g., `ProtocolProfile` detector, `SlotClock`, or `BankGraph`).
- Adhere to `AGENTS.md` and `.agents/rules/chrono-core.md`:
  - Zero-allocation structures where possible.
  - Pluggable interfaces without vendor-specific coupling.
  - Pure unit-testable domain logic.

## Step 6: Run Comprehensive Tests
- Execute automated unit tests for:
  - Happy path edge cases.
  - Malformed or out-of-order packet handling.
  - State invalidation on `UpdateParent` (for bank-aware modules).
- Assert 100% test passing before proceeding.

## Step 7: Run Benchmarks (If Applicable)
- If the module operates on a timing-critical path (e.g., `SlotClock`, `StreamNormalizer`, `BankGraph` insertion), run empirical profiling adhering to `docs/BENCHMARK_PROTOCOL.md`.
- Capture full distribution (p50, p90, p95, p99, Min, Max) over N ≥ 10,000 iterations.
- Explicitly label all results as `[MEASURED]`.

## Step 8: Update Documentation
- Update `docs/ARCHITECTURE.md` if any concrete interface details evolved.
- Update `docs/RESEARCH_REPORT.md` if new cluster behaviors were empirically observed.

## Step 9: Record Decisions in Decision Log
- If any architectural trade-offs, serialization formats, or internal buffering parameters were decided, record a new ADR entry in `docs/DECISIONS.md`.

## Step 10: Report Results to Lead Architect
- Present a concise, structured report detailing:
  - Module implemented.
  - Verification results and test output.
  - Empirical benchmark metrics (if applicable).
  - Open questions resolved or newly discovered.
  - Next module recommended for implementation.
