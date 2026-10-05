# CHRONO PHASE 6: LOCAL VALIDATOR & FULL-FIDELITY SETUP GUIDE

> **Document**: `docs/PHASE_6_VALIDATOR_SETUP.md`  
> **Status**: RATIFIED & EMPIRICALLY VERIFIED  
> **Scope**: Step-by-step instructions for launching a local Agave test validator with the Chrono Geyser plugin, establishing real UDS IPC streaming, and ingesting live validator telemetry at $0 budget.

---

## 1. Prerequisites & Environment

CHRONO is committed to the **$0 Budget First Principle**. All validator instrumentation runs entirely on local compute using open-source tools:
- **Agave / Solana CLI Suite**: `solana-cli 2.0.1+` (or `agave-validator 4.3.0+`)
- **Rust Toolchain**: `rustc 1.80+` / `cargo`
- **Operating System**: macOS (Darwin arm64/x86_64) or Linux

Verify the local validator binary:
```bash
/Users/prakhargaur/.local/share/solana/install/releases/2.0.1/solana-release/bin/solana-test-validator --version
# Output: solana-test-validator 2.0.1 (client:Agave)
```

---

## 2. Compiling the Chrono Geyser Plugin

Build the dynamic shared library:
```bash
cargo build -p chrono-geyser-plugin
```

This compiles:
- macOS: `target/debug/libchrono_geyser_plugin.dylib`
- Linux: `target/debug/libchrono_geyser_plugin.so`

---

## 3. Configuring the Plugin

Create `artifacts/config/chrono-geyser-plugin.json`:
```json
{
  "libpath": "/Users/prakhargaur/Desktop/CHRONO/target/debug/libchrono_geyser_plugin.dylib",
  "uds_socket_path": "/tmp/chrono_geyser.sock",
  "observer_validator_id": "LocalAgaveTestValidator",
  "enable_bank_status": true,
  "enable_block_footer": true,
  "enable_update_parent": true,
  "enable_deshred": true,
  "enable_entries": true,
  "channel_buffer_capacity": 65536
}
```

---

## 4. Starting Chrono Server in Local Validator Mode

Start the Chrono background daemon listening on the Unix Domain Socket:
```bash
cargo run -p chrono-cli -- serve --port 8900 --source local-validator --uds-path /tmp/chrono_geyser.sock
```

Chrono Server initializes the UDS listener at `/tmp/chrono_geyser.sock` and configures the `ProviderCapabilityMatrix` to **Level 4 (Validator Full-Fidelity)**.

---

## 5. Launching the Local Agave Validator

In a separate terminal or background task, launch `solana-test-validator` loading the Chrono Geyser plugin:
```bash
/Users/prakhargaur/.local/share/solana/install/releases/2.0.1/solana-release/bin/solana-test-validator \
  --reset \
  --quiet \
  --geyser-plugin-config /Users/prakhargaur/Desktop/CHRONO/artifacts/config/chrono-geyser-plugin.json
```

### What Happens:
1. Agave Validator initializes ledger in test ledger directory (`test-ledger/`).
2. Validator dynamic loader opens `libchrono_geyser_plugin.dylib` and calls `_create_plugin`.
3. The plugin connects to `/tmp/chrono_geyser.sock`.
4. As blocks and entries are generated, the validator invokes `update_bank_status`, `notify_entry`, and `notify_block_metadata`.
5. Raw events flow over the Unix socket directly into Chrono's `ValidatorGeyserSource` at microsecond latency.

---

## 6. Generating Real Validator Activity

To generate continuous transaction and entry activity for telemetry ingestion:
```bash
# Fund a keypair and send continuous transfers on local validator
solana airdrop 10 --url http://127.0.0.1:8899
solana transfer --url http://127.0.0.1:8899 --allow-unfunded-recipient <RECIPIENT_PUBKEY> 0.001
```

---

## 7. Inspecting Live Full-Fidelity Telemetry

Verify with Chrono CLI:
```bash
# Check telemetry source status & event counters
cargo run -p chrono-cli -- telemetry status

# Deep inspect all live telemetry fields
cargo run -p chrono-cli -- telemetry inspect

# Measure live coverage
cargo run -p chrono-cli -- telemetry coverage --local-validator
```

---

## 8. Mode Disambiguation Invariant

| Mode Name | Source Identifier | Nature | When Used |
|---|---|---|---|
| **Local Geyser Fixture** | `local-geyser-fixture` | Synthetic deterministic fixture playback | Unit testing, CI/CD regression without local validator binary |
| **Local Validator Full-Fidelity** | `local-validator` | Real Agave validator running live with dynamic plugin | Real-world telemetry verification, latency forensics, live bank graph validation |

> **CHRONO Master Invariant**: The UI and CLI must never report `local-geyser-fixture` as a live validator.
