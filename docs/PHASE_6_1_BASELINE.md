# CHRONO Phase 6.1 — Baseline Capture
> Generated: 2026-10-03T23:17:18Z  
> Purpose: Authoritative starting state before any Phase 6.1 changes.

---

## 1. Version & Toolchain

| Item | Value |
|---|---|
| **Rust toolchain** | `stable-aarch64-apple-darwin` |
| **rustc** | `1.99.0 (b940084d7 2026-09-28)` |
| **cargo** | `1.99.0 (5f94df478 2026-08-27)` |
| **Git revision** | No git repository (working directory without VCS) |
| **Solana CLI** | `solana-cli 2.0.1 (src:d02dac60; feat:4288794394, client:Agave)` |
| **solana-test-validator** | `2.0.1 (src:d02dac60; feat:4288794394, client:Agave)` |
| **Geyser Interface** | Agave 2.0.x GeyserPlugin trait (standard + Alpenglow 4.3 hooks modeled in chrono-geyser-plugin) |
| **Yellowstone** | yellowstone-grpc-proto = "13.0.0" per OQ-006; not actively connected at baseline |
| **Node.js** | v21.6.2 (at /usr/local/bin/node) |

---

## 2. Build Status

| Command | Result |
|---|---|
| `cargo build --workspace` | SUCCESS — 0 errors |
| `cargo build -p chrono-geyser-plugin --release` | SUCCESS — libchrono_geyser_plugin.dylib (411,200 bytes) |
| `cargo test --workspace` | SUCCESS — 54 tests, 0 failed |
| `cargo clippy --workspace --all-targets` | SUCCESS — 0 errors; warnings only |
| `tsc --noEmit` | SUCCESS — 0 TypeScript errors |

---

## 3. Failing Tests at Baseline

NONE. All 54 tests pass.

---

## 4. Clippy Warnings (No Errors)

- `this impl can be derived` — chrono-core
- `this function has too many arguments` — CertificateEngine, BankGraph, UDS adapter  
- `unsafe function's docs are missing a Safety section` — _create_plugin() in geyser lib.rs
- `field assignment outside of initializer for an instance created with Default::default()` — multiple
- `large size difference between variants` — ChronoEventKind enum
- `accessing first element with leaders.get(0)` — leader scheduling

No warnings prevent correctness or compilation.

---

## 5. Previously Known Phase 6 Issues — Current Status

| Issue | Status |
|---|---|
| A. EvidenceGraph source-authority mismatch | NOT REPRODUCING — Geyser authority rule works correctly |
| B. First bank not ABANDONED after UpdateParent | FIXED — mark_abandoned_by_id() called in state.rs |
| C. replacement_bank_id = None | FIXED — pending_replacement tracked in UDS adapter |
| D. Schema/API type mismatches | NOT REPRODUCING — all types compile consistently |
| Double-counted certificates | FIXED — only CertificateObserved path pushes to captured_certificates |
| bank_hash None on bank2 | FIXED — attach_footer() called in BlockFooterObserved handler |

---

## 6. Test Classification

| Test | Classification | Can Prove Real Validator? |
|---|---|---|
| test_geyser_ipc_framing_and_transmission | INTEGRATION — IPC (synthetic frames, real UDS) | NO |
| test_geyser_ipc_end_to_end_telemetry_ingestion | INTEGRATION — FIXTURE (synthetic frames, real UDS, real state machine) | NO |
| test_multi_source_conflict_reconciliation | UNIT — FIXTURE | NO |
| test_runtime_capability_matrix_diagnostics | UNIT — FIXTURE | NO |

No test uses a real Agave validator at baseline.

---

## 7. Validator Runtime Status

| Item | Status |
|---|---|
| solana-test-validator binary | PRESENT: ~/.local/share/solana/install/active_release/bin/solana-test-validator |
| Validator currently running | NOT RUNNING |
| Plugin loaded | NOT LOADED |
| IPC connected | NOT CONNECTED |
| Real events observed | NOT YET |

---

## 8. Plugin Artifact

| Item | Value |
|---|---|
| Plugin dylib | /Users/prakhargaur/Desktop/CHRONO/target/release/libchrono_geyser_plugin.dylib |
| Plugin size | 411,200 bytes |
| Crate type | cdylib + rlib |
| C ABI entry | _create_plugin() |
| Default UDS socket | /tmp/chrono_geyser.sock |

---

## 9. Remaining Phase 6.1 Work

1. Runtime validation — run real validator + plugin, capture real events, replay, prove determinism
2. New regression tests — failure injection, ordering edge cases, footer-before-bank, duplicate cert
3. Code quality — fix clippy warnings (Safety doc, get(0), Default initializer)
4. Final report — PHASE_6_1_FINAL_REPORT.md and test matrix
