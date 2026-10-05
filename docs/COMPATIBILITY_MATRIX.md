# CHRONO Core Compatibility Matrix

> **Document Status**: Production Complete  
> **Schema Version**: `chrono-v1.0.0`  
> **Last Verified**: Phase 7 Final

---

## 1. Upstream Protocol & Software Compatibility

CHRONO Core is validated against active, feature-gated, and proposed Solana / Agave protocol versions:

| Component | Target Version | Supported Versions | Compatibility Status | Notes |
| :--- | :---: | :---: | :---: | :--- |
| **Agave Validator** | `v4.3.0` | `v2.2.0` – `v4.3.0` | **FULL** | Compatible with standard RPC and v4.3.0 Votor / Alpenglow structures. |
| **Solana CLI Toolchain** | `v2.2.0` | `v2.0.0` – `v2.2.0` | **FULL** | Local validator testbed (`solana-test-validator`). |
| **Yellowstone gRPC** | `v13.0.0` | `v10.0.0` – `v13.0.0` | **FULL** | Protobuf schema compatibility for slots, blocks, and transactions. |
| **Geyser Plugin ABI** | `v4.3.0` | `v2.1.0` – `v4.3.0` | **FULL** | Implements standard C ABI `on_load`, `notify_block`, `notify_slot`. |
| **Solana BLS Signatures** | `v3.4.0` | `v3.0.0` – `v3.4.0` | **FULL** | Underlying BLST engine for genuine BLS12-381 certificate verification. |
| **Rust Compiler** | `1.85.0` | `>=1.80.0` | **FULL** | Compiles with `-D warnings` on all targets and features. |
| **Node.js Toolchain** | `v22.23.1` | `>=v20.0.0` | **FULL** | Next.js 16.0.10 production build verified. |

---

## 2. Protocol Feature Gating & SIMD Matrix

| Protocol Specification | Status in Upstream | CHRONO Handling | Behavior if Inactive |
| :--- | :---: | :---: | :--- |
| **SIMD-0326 (Alpenglow)** | In Development / Testnet | `ProtocolMode::Alpenglow` | Falls back to single-bank slot tracking (`LegacyTowerBFT`). |
| **SIMD-0298 (Block Footer)** | Feature-Gated | `BlockFooter` Parser | Evaluates standard block without footer; bank hash from RPC. |
| **SIMD-0337 (UpdateParent)** | Feature-Gated | Fast Handover Engine | Follows linear parent links without candidate bank pruning. |
| **SIMD-0384 (Migration)** | Proposed / Staged | Dynamic Mode Engine | Detects feature activation slot and transitions state machine. |
| **Rotor / Smart Relayers** | In Development | Deferred | Live clusters use standard **Turbine** propagation. |

---

## 3. Startup Compatibility & Version Enforcement

At startup, CHRONO Core inspects upstream connection endpoints and asserts compatibility:

1. **Protocol Mode Check**: Queries cluster feature set. If Alpenglow features are detected, protocol mode initializes to `Alpenglow`. Otherwise, initializes to `LegacyTowerBFT`.
2. **Schema Incompatibility**: If an upstream provider sends payloads with an unrecognized schema version or malformed framing, the event is quarantined and logged. Core state machine continuity is preserved.
3. **Critical Rejection**: CHRONO rejects startup if the configured Agave Geyser C-ABI version is incompatible with the loaded plugin shared object.
