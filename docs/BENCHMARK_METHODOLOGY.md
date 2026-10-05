# CHRONO Benchmark Methodology & Latency Integrity Protocol

> **Document**: `docs/BENCHMARK_METHODOLOGY.md`  
> **Status**: Verified Operational Protocol  
> **Subsystem**: `crates/chrono-bench` & `chrono bench` CLI

---

## 1. Metric Classification Standard

Every latency figure, timestamp, or timing assertion generated in CHRONO Phase 4 MUST be classified under one of the four immutable tiers:
- `[MEASURED]`: Empirically recorded from monotonic hardware clocks (`std::time::Instant` or Linux `CLOCK_MONOTONIC`) during execution against running processes or network endpoints.
- `[ESTIMATED]`: Analytically calculated from verified protocol parameters (e.g. cluster slot duration models, speed-of-light propagation bounds).
- `[SIMULATED]`: Generated in deterministic synthetic environments or local mock harnesses.
- `[UNKNOWN]`: Unverified, unobservable, or missing from provider telemetry.

**Absolute Directive**: Simulated metrics can NEVER be cited or presented as evidence of live network latency.

---

## 2. The 11-Stage Atomic Latency Breakdown

Chrono rejects vague "execution latency" aggregates. Every sample recorded by `BenchmarkRunner` decomposes the transaction lifecycle into 11 discrete monotonic hardware timestamps:

```text
T0 (Decision Available) ───[decision_to_build]───> 
T1 (Template Build Start) ───[build_to_sign]───────> 
T2 (Fully Signed) ───────────[sign_to_submit]──────> 
T3 (Submission Request) 
T4 (Dispatched to Route) ────[submit_to_ack]───────> 
T5 (Route Acknowledged) ────[submit_to_obs]───────> 
T6 (Observed on Solana) ────[submit_to_landed]────> 
T7 (Landed in Slot) 
T8 (Processed by Leader) ───[submit_to_confirmed]─> 
T9 (Confirmed by Stake) ────[submit_to_finalized]─> 
T10 (Finalized / Rooted)
```

### Stage Definitions & Clock Monotonicity:
1. **T0 (Decision Available)**: The exact microsecond the routing state machine evaluates current slot progress, leader identity, and bank state.
2. **T1 (Template Build Begins)**: Zero-allocation Solana transfer transaction serialization begins.
3. **T2 (Fully Signed)**: High-performance `ed25519-dalek` signature completed (< 15µs).
4. **T3 (Submission Request Begins)**: Dispatched to the `ExecutionRoute` implementation.
5. **T4 (Leaves Chrono)**: Network payload transmitted over wire.
6. **T5 (Route Acknowledged)**: Transport RTT receipt confirmed by the receiving RPC/TPU node.
7. **T6 (First Network Observation)**: Transaction signature first detected on Solana gossip/shred stream.
8. **T7 (Landed in Slot)**: Transaction included in a produced candidate or canonical bank.
9. **T8 (Processed)**: BankingStage transaction execution completed.
10. **T9 (Confirmed)**: Cluster achieves 66%+ stake confirmation or Votor fast-path notarization.
11. **T10 (Finalized)**: BLS finalization certificate notarized or TowerBFT 32-slot lockout reached.

---

## 3. Interleaved A/B Testing Discipline

To eliminate temporal bias (such as background internet congestion, validator stalls, or slot rate fluctuations during sequential runs), Chrono benchmarks use **strictly interleaved trial assignment**:

```text
Trial Index:    0    1    2    3    4    5    6    7    ...
Assigned Group: C    H    C    H    C    H    C    H    ...
Where:
C = CONTROL (Standard RPC client baseline)
H = CHRONO-AWARE (Chrono consensus-aware routing engine)
```

Both groups run under identical conditions:
- Same signer keypair type (`ed25519`)
- Same transaction payload (deterministic low-risk SOL transfer)
- Same cluster target (Local fixture or Testnet)
- Same approximate time window
- Same network interface

---

## 4. Environment Stratification

Results from different execution environments must NEVER be aggregated into a single blended metric. Benchmarks are strictly stratified:
- **LOCAL**: Deterministic, high-throughput in-memory fixture tests ($0 budget, N >= 1,000 samples). Used to benchmark CPU routing overhead, state machine decision latency, and candidate bank invalidation logic.
- **TESTNET**: Live Solana Testnet RPC execution (N >= 20–100 samples). Used to measure real-world network RTT, signature status polling, and public RPC rate limits.
- **DEVNET**: Fallback public cluster.
- **MAINNET**: Observation and telemetry only; benchmark write traffic is strictly prohibited on Mainnet.

---

## 5. Signer Safety & Privacy Constraints

In strict accordance with Core Engineering Rules:
- Private keys and secret seeds are **never logged**, printed, or committed to disk.
- Test signers use ephemeral, in-memory generated keypairs or explicit test keys.
- Benchmark artifacts (`samples.jsonl`) record ONLY public keys, transaction signatures, execution IDs, and timestamps.

---

## 6. Artifact Storage Schema

Benchmark runs are stored under `artifacts/benchmarks/<exp_id>/` with four immutable files:
1. `experiment.json`: Complete configuration parameters, seed, cluster, and route identities.
2. `samples.jsonl`: Line-delimited raw JSON records for every attempted sample.
3. `summary.json`: Statistical aggregate metrics, percentiles (p50, p90, p95, p99), distributions, and verdict.
4. `README.md`: Human-readable comparative markdown report formatted for immediate operator review.

---

## 7. Funding Preflight & Safe Wallet Protocol (Phase 4.1 Addition)

Live cluster benchmark execution requires verifiable funding for transaction fees (5,000 lamports / signature + safety margin). To guarantee experimental integrity and prevent false-negative runs:

1. **Preflight Verification**: Prior to trial 0, `chrono bench preflight` calculates the required budget:
   $$\text{Required Lamports} = (\text{Samples} \times 6,000) + 10,000$$
2. **Safe Signer Hierarchy**:
   - Explicit CLI path (`--keypair <path>`)
   - Environment variable (`CHRONO_KEYPAIR` or `TESTNET_KEYPAIR`)
   - Solana CLI configuration (`~/.config/solana/id.json`)
   - Ephemeral in-memory keypair (isolated to test runtime)
3. **Hard Abort on Insufficient Funds**: If available balance is below required budget and automatic bounded faucet requests fail, the benchmark runner immediately aborts with `INSUFFICIENT_FUNDS`. It never silently begins an experiment guaranteed to fail.
4. **Zero Secret Leakage**: Private keys, seed phrases, and mnemonics are strictly prohibited from logging, telemetry, JSONL records, error payloads, and terminal outputs.

---

## 8. Invalid Run Handling & Historical Data Exclusion

When an experimental trial fails due to infrastructure preconditions rather than system execution performance (such as an unfunded wallet resulting in 0% submission success), the run MUST be handled under strict scientific protocols:
- **Status Flagging**: Formally marked as `INVALID_UNFUNDED_RUN` or equivalent historical status.
- **Exclusion Invariant**: Never merged into successful sample sets or cited as proof of execution advantage or disadvantage.
- **Immutability**: Raw trial records (`samples.jsonl`) remain preserved for forensic auditability and root cause attribution.

---

## 9. Execution Success vs Submission Acknowledgment

CHRONO enforces a strict separation between transport acceptance and consensus execution:
- **`SUBMITTED`**: The RPC acknowledged HTTP POST receipt and returned a transaction signature. *This is not execution success.*
- **`OBSERVED`**: The transaction was detected in validator shred streams or gossip.
- **`LANDED`**: The transaction was included in an executed bank at a specific slot.
- **`CONFIRMED` (Primary Success Benchmark)**: The transaction achieved 66%+ cluster stake confirmation.
- **`FINALIZED` (Secondary Success Benchmark)**: The transaction reached cryptographic finality (32 TowerBFT lockout roots or Votor BLS finalization certificates).

---

## 10. Failure Taxonomy Standard

Every non-successful trial is classified into a deterministic error category:
- `INSUFFICIENT_FUNDS`: Account balance below fee requirement.
- `BLOCKHASH_EXPIRED`: Blockhash was not found in recent blockhash queue.
- `BLOCKHASH_NOT_FOUND`: Blockhash unknown to RPC node.
- `RPC_TIMEOUT`: Polling exceeded configured threshold (`timeout_ms`).
- `RPC_REJECTED`: Preflight simulation rejected transaction logic.
- `TRANSACTION_ERROR`: On-chain program execution failed.
- `SOURCE_STALE`: Chrono ingestion source degraded or stale.
- `SIGNING_ERROR`: Keypair signature verification failure.

Failed samples are NEVER discarded or filtered out of statistical success rate calculations.

---

## 11. Rate Limit & Bounded Cluster Budget Policy

To protect public cluster health and prevent RPC throttling:
- Default transaction rate is capped at `1.0 TPS`.
- Experiments are strictly bounded: planned samples typically $N = 50 \text{ to } 100$ per run.
- Self-transfers (`sender == recipient`) eliminate rent-exemption creation costs while testing full consensus, banking stage, and signature verification pathways.

---

## 12. Statistical Analysis & Four Allowed Verdicts

All comparative evaluations between Control and Chrono-Aware pathways must culminate in exactly one of four objective, data-backed verdicts:
1. **`CONFIRMED ADVANTAGE`**: Both groups achieve high sample counts, interleaved trials, and Chrono demonstrates statistically significant latency reduction (p50/p95) or higher confirmation rates under identical transport conditions.
2. **`CONDITIONAL ADVANTAGE`**: Chrono provides measurable benefits specifically under bounded edge conditions (e.g. leader handoff boundaries, bank invalidation via `UpdateParent`, or stale blockhashes), without broad baseline separation.
3. **`NO MATERIAL ADVANTAGE`**: Both pathways exhibit statistically indistinguishable execution latencies and confirmation rates within normal cluster jitter margins.
4. **`INCONCLUSIVE`**: Insufficient valid samples, cluster outages, rate-limiting, or transport failures prevent statistically rigorous conclusions.

