# CHRONO Benchmark Protocol & Latency Standards

> **Core Mandate**: In God we trust, all others must bring data. Simulated numbers can NEVER be used as proof of real-world speed. Every benchmark result must be strictly reproducible, empirical, and statistically rigorous.

---

## 1. Metric Classification Taxonomy

Every timing value, latency estimate, or benchmark report in CHRONO must explicitly carry one of four classifications:

1. **`MEASURED`**:
   - Recorded directly from hardware timestamp counters (`Instant::now()` monotonic clock / TSC) during live or recorded packet execution over real network interfaces or local sockets.
   - Requires published sample size, cluster environment, and hardware specifications.

2. **`ESTIMATED`**:
   - Analytically derived using verified mathematical formulas based on measured lower bounds (e.g., speed of light in fiber over geographical distance, theoretical minimum network hops).
   - Must explicitly list all assumptions and input parameters.

3. **`SIMULATED`**:
   - Generated in synthetic test harnesses, mocked networks, or offline deterministic unit tests.
   - **EXPLICIT PROHIBITION**: Simulated numbers are strictly forbidden from being cited as proof of real-world speed or competitive advantage. They may only be used for regression testing and logical assertion.

4. **`UNKNOWN`**:
   - Data has not yet been empirically gathered or verified under current cluster conditions.

---

## 2. Hardware Clock Source & Timestamping Rules

1. **Monotonic High-Resolution Clock**:
   - Benchmarking must exclusively use monotonic clock sources immune to NTP adjustments, daylight savings shifts, or system clock slewing (e.g., `clock_gettime(CLOCK_MONOTONIC_RAW)` or CPU Time Stamp Counter `rdtsc` where serialized).
2. **Nanosecond Precision**:
   - All internal timestamps must be captured as `u64` nanoseconds since epoch start or benchmark session initialization.
3. **Capture Overheads**:
   - Benchmarking probes must introduce zero or minimal overhead (< 15ns per probe).
   - In hot paths, timestamps are captured into pre-allocated circular buffers, never formatted or logged synchronously.

---

## 3. Core Latency Metrics Defined

| Metric | Definition | Start Event | Stop Event |
|---|---|---|---|
| **Detection Latency** | Time to detect an on-chain event or slot boundary | Byte arrives at network socket buffer | Parsed event emitted to internal Event Bus |
| **Decision Latency** | Engine processing & strategy evaluation time | Event Bus delivery to strategy engine | Execution action decision emitted |
| **Build Latency** | Transaction serialization & instruction assembly | Decision emitted | Raw transaction bytes serialized in memory |
| **Signing Latency** | Cryptographic signature generation | Byte serialization complete | Ed25519 signature verified on payload |
| **Send Latency** | Network dispatch latency | Signature verified | Last byte flushed to network socket (QUIC/UDP) |
| **Time to Leader** | Physical transit time to current slot leader | Socket flush | Packet arrival at leader's TPU interface |
| **Time to Land** | Inclusion latency into candidate block | Packet arrival at leader | Inclusion into candidate bank shred stream |
| **Time to Canonical** | Duration until candidate bank is confirmed canonical | Candidate bank first observed | Blockhash/bank confirmed canonical by resolver |
| **Time to Finality** | Duration until block is irrevocably finalized | Candidate bank first observed | BLS finality certificate (`block_final_cert`) or 32 lockouts verified |
| **End-to-End Latency** | Total elapsed pipeline latency | Trigger event arrival at network interface | Irrevocable finality confirmation of response transaction |

---

## 4. A/B Testing Methodology

To prove optimization without bias:
1. **Control vs Treatment**:
   - Legacy baseline path (e.g., standard polling or naive single-bank WebSocket) runs concurrently alongside CHRONO Alpenglow-aware bank graph path under identical network conditions and hardware.
2. **Identical Stream Ingestion**:
   - Both pipelines consume the exact same physical byte stream via tee/fanout to eliminate network jitter variance.
3. **Statistical Sample Sizes**:
   - Minimum sample size for any published benchmark: **N = 10,000 slots or events**.
   - No micro-benchmarks of < 1,000 iterations may be quoted as representative.
4. **Required Statistical Distribution**:
   Every benchmark report must present the full distribution:
   - **Min**: Absolute minimum observed
   - **p50 (Median)**: 50th percentile
   - **p90**: 90th percentile
   - **p95**: 95th percentile
   - **p99**: 99th percentile
   - **Max**: Worst-case tail latency
   - **Failure / Drop Rate**: Percentage of dropped, timed-out, or invalidated events

---

## 5. Environment & Reproducibility Standards

Every benchmark document must record:
- **Cluster**: Devnet, Testnet, Localnet, or Mainnet-Beta.
- **Cluster Epoch & Slot Range**: Exact range during which data was collected.
- **Software Version**: Agave validator version, Yellowstone proto version, CHRONO commit hash.
- **Hardware Profile**: CPU model, core frequency, RAM speed, OS kernel version, NIC specifications.
- **Network Profile**: Datacenter region, measured ping to RPC/validator, packet drop rate during test.
