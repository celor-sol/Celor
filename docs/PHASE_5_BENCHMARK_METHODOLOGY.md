# CHRONO Phase 5: Benchmark Methodology & Experimental Design

## 1. Objective & Hypothesis

The Phase 4.1 live experiment demonstrated that over public JSON-RPC transport, Chrono-Aware consensus timing was masked by RPC gateway queuing.

**Phase 5 Core Hypothesis**:
> Direct leader QUIC transport removes public RPC queuing delay, allowing Chrono's slot-boundary lookahead, blockhash freshness, and candidate bank awareness to translate into superior landing latency and higher confirmation rates.

---

## 2. The 2x2 Factorial Experimental Matrix

To isolate the causal effects of **Transport Layer** (RPC vs QUIC) and **Consensus Awareness** (Control vs Chrono-Aware), Phase 5 employs a 2x2 factorial experimental design:

| Route ID | Consensus Awareness | Transport Layer | Mechanism Description |
| :--- | :--- | :--- | :--- |
| **Route A** | Control | JSON-RPC | Standard Solana client: blind submission, public RPC proxy, generic blockhash. |
| **Route B** | Chrono-Aware | JSON-RPC | Chrono timing & blockhash freshness, dispatched through public RPC proxy. |
| **Route C** | Control | Direct QUIC | Blind submission directly to leader's TPU QUIC socket (no slot boundary awareness). |
| **Route D** | Chrono-Aware | Direct QUIC | Chrono timing, leader lookahead, dual-dispatch, directly to leader's TPU QUIC socket. |

### 2.1 Causal Effect Decompositions
By comparing these four cells, we isolate four distinct effects:

1. **Transport Effect (Baseline)**:
   $$\Delta_{\text{Transport}} = \text{Metric}(\text{Route C}) - \text{Metric}(\text{Route A})$$
   *Measures the pure impact of switching from RPC to QUIC without any consensus intelligence.*

2. **Chrono Effect on RPC**:
   $$\Delta_{\text{Chrono, RPC}} = \text{Metric}(\text{Route B}) - \text{Metric}(\text{Route A})$$
   *Replicates the Phase 4.1 experiment.*

3. **Chrono Effect on QUIC**:
   $$\Delta_{\text{Chrono, QUIC}} = \text{Metric}(\text{Route D}) - \text{Metric}(\text{Route C})$$
   *Measures whether Chrono awareness provides an advantage when using direct QUIC transport.*

4. **Total System Effect**:
   $$\Delta_{\text{Total}} = \text{Metric}(\text{Route D}) - \text{Metric}(\text{Route A})$$
   *Measures the full transformation from standard RPC client to Chrono Direct QUIC.*

---

## 3. Experimental Controls & Rigor

### 3.1 Interleaved Round-Robin Scheduling
To eliminate temporal bias (e.g., cluster congestion spikes, sudden fee surges, network partitions), samples are **never run in sequential batches**. Instead, execution cycles through the 4 routes in strict round-robin order:

$$\text{Sample } 0 \to \text{Route A}, \quad \text{Sample } 1 \to \text{Route B}, \quad \text{Sample } 2 \to \text{Route C}, \quad \text{Sample } 3 \to \text{Route D}, \quad \dots$$

Every route experiences statistically identical network conditions over time.

### 3.2 Monotonic Hardware Clocking
All epoch timestamps (T0 through T10) are recorded using monotonic CPU hardware clocks (`std::time::Instant`). Wall-clock time (`SystemTime`) is used solely for audit logs.

### 3.3 Metric Classification Standards
In accordance with CHRONO master rules:
- `[MEASURED]`: Monotonic hardware clock timings recorded during live cluster execution.
- `[ESTIMATED]`: Analytically computed network RTTs or slot duration offsets.
- `[SIMULATED]`: Pure mock or synthetic fixtures (never claimed as live network performance).

---

## 4. Failure & Error Taxonomy

Every failed execution is classified into a mutually exclusive failure category:

| Error Category | Diagnostic Criteria | Remediation / Interpretation |
| :--- | :--- | :--- |
| **`StaleBlockhash`** | Transaction rejected due to expired or invalid blockhash. | Avoidable via Chrono blockhash freshness tracking. |
| **`LeaderHandoffMiss`** | Dispatched to leader whose slot boundary elapsed before inclusion. | Mitigated via Chrono slot-tail suppression / lookahead. |
| **`QuicHandshakeTimeout`**| Direct QUIC TLS 1.3 handshake timed out (>2,000ms). | Network partition or firewall drop; triggers fallback if enabled. |
| **`SocketUnreachable`** | Leader endpoint not advertising active `tpuQuic` or invalid IP. | Resolved via `LeaderTransportResolver`. |
| **`ConfirmationTimeout`**| Transaction submitted but not confirmed within poll limit (30s). | Cluster congestion or validator dropped transaction from QOS. |
| **`InsufficientFunds`** | Keypair fee balance insufficient to pay base fee. | Pre-execution balance check prevents invalid runs. |

---

## 5. Decision Rules for Empirical Verdict

The final benchmark report must conclude with exactly one of four verdicts:

1. **`CONFIRMED ADVANTAGE`**:
   Chrono+QUIC (Route D) outperforms Control+QUIC (Route C) and Control+RPC (Route A) on **both** confirmation rate ($\ge +5\%$) and median confirmation latency ($\le -200\text{ms}$) with statistical significance ($p < 0.05$).

2. **`CONDITIONAL ADVANTAGE`**:
   Chrono+QUIC demonstrates clear superiority under specific measurable conditions (e.g., during slot-tail boundaries or high leader handover volatility), but exhibits parity or marginal difference during mid-slot steady state.

3. **`NO MATERIAL ADVANTAGE`**:
   Neither Chrono-aware timing nor QUIC transport produces a statistically significant improvement over baseline control routes.

4. **`INCONCLUSIVE`**:
   Network anomalies, cluster outages, or sample size limitations ($N < 100$ per cell) prevent definitive statistical assertions.
