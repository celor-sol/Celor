# AGENTS.md — CHRONO Master Directives

> **Project**: CHRONO — Provider-Independent Solana Consensus & Timing Infrastructure  
> **Status**: Architecture & Research Bootstrap (No production application code implemented yet)  
> **Initial Budget Constraint**: $0 (Devnet/Testnet & Free Tier Infrastructure First)

---

## 1. Core Operating Principles

1. **Protocol Truth Over Assumptions**:
   - Never invent protocol behavior, fields, or RPC interfaces.
   - Verify current Solana behavior against Agave source code, SIMDs, and official Solana changelogs before specifying or writing any code.
   - Official primary sources (Solana Foundation SIMDs, Anza/Agave repo, Solana Docs) strictly take precedence over third-party blog posts, social posts, or speculative articles.
   - Classify all protocol features strictly: `LIVE`, `FEATURE-GATED`, `IN DEVELOPMENT`, `PROPOSED`, `FUTURE`, or `UNKNOWN`. Never blur these boundaries.

2. **Benchmarking & Latency Integrity**:
   - Never fabricate, assume, or simulate benchmark results to claim real-world performance.
   - Every timing figure must be explicitly labeled: `MEASURED`, `ESTIMATED`, `SIMULATED`, or `UNKNOWN`.
   - Simulated metrics can NEVER be used as evidence of live network latency.
   - Every performance optimization requires an empirical baseline and comparative measurement (A/B testing under identical conditions).

3. **Alpenglow & Bank Architectural Reality**:
   - **Do not assume `slot == block`**: Under Alpenglow (SIMD-0326 / SIMD-0337 / Agave 4.3+), a single slot can contain multiple candidate banks.
   - **`bank_id` is validator-local**: It is unique ONLY within a specific validator node's internal state. It is NOT globally unique across the Solana cluster.
   - **Block identity / blockhash is canonical**: Use `blockhash` or deterministic block identity for cross-provider and cross-validator reconciliation.
   - **Observed ≠ Canonical ≠ Finalized**:
     - *Observed*: Received via shred, gRPC, or WebSocket from a provider.
     - *Canonical*: Resolved on the longest valid chain / confirmed fork for that slot.
     - *Finalized*: Cryptographically certified by Votor notarization/finalization certificates (fast path 80% stake, fallback path 60% stake) or legacy TowerBFT root.
   - **Never assume Rotor is live**: Rotor is in development / proposed; block propagation on live clusters continues to use Turbine.

4. **Provider Independence**:
   - CHRONO must never become vendor-locked to Helius, Triton, Jito, or any single vendor.
   - All external feeds must adapt through standardized, swappable Provider Adapters.
   - Multi-provider stream normalization and conflict resolution must be core system invariants.

5. **Resource & Development Discipline**:
   - **$0-First Constraint**: Initial development, testing, and validation must run entirely on Solana Devnet/Testnet and free-tier infrastructure.
   - **Preserve Legacy Control Paths**: Keep legacy slot/block processing logic intact alongside Alpenglow paths to enable rigorous A/B performance baselines.
   - **No Unnecessary Rewrites**: Only write or refactor code when backed by verified protocol specifications and empirical profiling.
