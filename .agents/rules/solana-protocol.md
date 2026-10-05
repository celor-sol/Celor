---
trigger: always_on
description: Solana protocol invariants, Alpenglow consensus reality, and bank handling rules
---

# Solana Protocol & Alpenglow Invariants

## 1. Protocol Categorization Requirement
Every protocol assertion or implementation branch must be explicitly categorized:
- `LIVE`: Active on Solana Mainnet-Beta today.
- `FEATURE-GATED`: Merged into validator clients, gated by cluster feature activation.
- `IN DEVELOPMENT`: Active in test environments (Devnet/Testnet) or under development in Agave/Firedancer.
- `PROPOSED`: Formalized in a SIMD but not yet implemented or deployed.
- `FUTURE`: Long-term architectural vision.
- `UNKNOWN`: Status uncertain; requires verification.

## 2. Invariants Regarding Slots and Banks
- **Do not assume `slot == block`**: Under Alpenglow (SIMD-0326 / SIMD-0337), multiple candidate banks can exist per slot.
- **`bank_id` is validator-local**: It is assigned internally by an Agave validator node. Never use `bank_id` as a globally unique identifier across providers or across validators.
- **Reconciliation standard**: Reconcile events across providers using `(slot, blockhash)` and parent lineage, never `bank_id`.
- **Observed ≠ Canonical ≠ Finalized**:
  - `Observed`: Seen on a stream.
  - `Canonical`: Confirmed on the winning fork.
  - `Finalized`: Cryptographically notarized via BLS certificates (Fast Path ~100ms, Fallback ~150ms) or 32 legacy lockouts.

## 3. Fast Leader Handover & State Invalidation
- When an `UpdateParent` marker is received, immediately invalidate, roll back, or purge all uncommitted transactions and state associated with the abandoned parent candidate bank.
- Never execute or route transactions against an uncommitted candidate bank without checking for parent invalidation.

## 4. No Premature Rotor Assumptions
- Rotor is in development and deferred from the initial Votor rollout.
- The live block propagation mechanism on all active Solana clusters is **Turbine**.
- Do NOT write code that assumes Rotor or Smart Sampling relayers are live today.
