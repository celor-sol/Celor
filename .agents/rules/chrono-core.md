---
trigger: always_on
description: Core architectural principles and engineering constraints for CHRONO
---

# CHRONO Core Engineering Rules

## 1. Provider Independence
- NEVER couple core business logic, domain entities, or state management to a specific external provider API (Helius, Triton, Jito, QuickNode, etc.).
- All data ingestion must pass through abstract `ProviderAdapter` interfaces.
- The internal representation of slots, banks, blocks, and events must be completely vendor-neutral.

## 2. $0 Budget Development Constraint
- All initial development, testing, and continuous integration must be executable with $0 capital expenditure.
- Rely exclusively on open-source software, local test validators (`solana-test-validator`), and free/public Devnet/Testnet infrastructure.
- Never write code or tests that fail or crash if commercial API keys are omitted.

## 3. High-Performance Engineering Discipline
- Prioritize zero-allocation data structures, pre-allocated ring buffers, and cache-friendly layout in hot paths.
- Avoid synchronous blocking operations or unbounded queue allocations on event-handling paths.
- Every architectural optimization must be motivated by measured bottlenecks, never speculation.

## 4. Documentation & Verification Integrity
- Maintain documentation in lockstep with architectural decisions.
- Any decision altering data flow, component boundaries, or external dependencies must be recorded in `docs/DECISIONS.md`.
- Never invent missing APIs, fields, or protocol behavior. If an interface is uncertain, mark it as `UNKNOWN` and log it in `docs/OPEN_QUESTIONS.md`.
