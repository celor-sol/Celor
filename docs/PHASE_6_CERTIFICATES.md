# CHRONO PHASE 6: ALPENGLOW CERTIFICATE SUBSYSTEM SPECIFICATION

> **Document**: `docs/PHASE_6_CERTIFICATES.md`  
> **Status**: RATIFIED & CRYPTOGRAPHICALLY FORMALIZED  
> **Scope**: Specification for raw certificate ingestion, structural parsing, BLS12-381 pairing verification, and finality engine correlation.

---

## 1. Overview of Alpenglow Consensus Certificates

Under Alpenglow (SIMD-0326), legacy on-chain vote transactions and 32-slot TowerBFT lockouts are replaced by **cryptographic aggregate certificates**. Thousands of individual validator signatures are compressed into a single Boneh-Lynn-Shacham (BLS12-381) aggregate signature, yielding deterministic single-slot finality.

Four certificate types exist in the Agave protocol:
1. **`BlockFinalizationCert`**: Certifies irreversible block finality.
   - Fast Path: $\ge 80\%$ active stake in a single round (~100ms - 150ms).
   - Fallback Path: $\ge 60\%$ active stake across two rounds (~150ms - 250ms).
2. **`NotarRewardCertificate`**: Attests that a block received notarization votes, used for epoch validator reward distribution.
3. **`SkipRewardCertificate`**: Attests that a leader was skipped and eligible voters chose to skip the slot.
4. **`GenesisCertificate` (`AgGenesisCert`)**: Root trust anchor certifying cluster initialization parameters (active on Devnet/Testnet).

---

## 2. In-Memory Agave Representation

From `agave-geyser-plugin-interface::block_footer`:

```rust
#[repr(C)]
pub struct VotesAggregate<'a> {
    pub signature: SignatureCompressed, // 96 bytes (G1) or 192 bytes (G2)
    pub bitmap: &'a [u8],               // Bitfield indicating which validators signed
}

#[repr(C)]
pub struct BlockFinalizationCert<'a> {
    pub slot: Slot,
    pub block_id: Hash,
    pub final_aggregate: VotesAggregate<'a>,
    pub notar_aggregate: Option<VotesAggregate<'a>>,
}
```

---

## 3. The Three Verification Tiers

To eliminate fabricated speed claims or false security assumptions, Chrono enforces three distinct certificate processing states:

```
┌─────────────────────────────────┐
│     RAW CERTIFICATE OBSERVED    │
│  - Raw wire bytes ingested      │
│  - Length & source validated    │
│  - NO parsing performed yet     │
└────────────────┬────────────────┘
                 │
                 ▼
┌─────────────────────────────────┐
│       CERTIFICATE PARSED        │
│  - Slot, Block ID extracted     │
│  - Bitmap decoded (signers)     │
│  - Aggregate pubkey computed    │
│  - Stake percentage calculated  │
│  - NO pairing verification yet  │
└────────────────┬────────────────┘
                 │
                 ▼
┌─────────────────────────────────┐
│    CRYPTOGRAPHICALLY VERIFIED   │
│  - BLS12-381 pairing evaluated: │
│    e(σ, g2) == e(H(m), pk_agg)  │
│  - Stake threshold verified     │
│  - Mathematically proven        │
└─────────────────────────────────┘
```

> **MASTER DIRECTIVE**: A parsed certificate MUST NEVER be labeled `VERIFIED` without successfully computing the cryptographic pairing check against the active cluster stake table.

---

## 4. BLS12-381 Pairing Verification Mechanics

For a candidate block with message $m = (\text{slot}, \text{block\_id})$:
1. **Message Hashing**:
   Map $m$ to a curve point in $G_1$ using hash-to-curve (RFC 9380):
   $$H(m) \in G_1$$
2. **Aggregate Public Key Computation**:
   Using the decoded bitfield $B$, retrieve the public key $pk_i \in G_2$ for each validator $i$ where $B[i] = 1$:
   $$pk_{\text{agg}} = \sum_{i \in \text{signers}} pk_i$$
3. **Bilinear Pairing Check**:
   Evaluate the Tate/Weil pairing over the BLS12-381 curve:
   $$e(\sigma, g_2) \stackrel{?}{=} e(H(m), pk_{\text{agg}})$$
   If the equality holds and $\sum \text{stake}_i \ge \text{Threshold}$, the certificate is promoted to `CRYPTOGRAPHICALLY_VERIFIED`.

---

## 5. Correlation with BankGraph & FinalityEngine

When a valid certificate is associated with an active bank:
1. `CertificateEngine` matches `(slot, block_id)` against the `BankGraph`.
2. If `block_id` matches an uncommitted candidate bank:
   - That candidate bank is immediately promoted to `CanonicalState::Finalized`.
   - All competing candidate banks for that slot are marked `CanonicalState::Dead`.
3. `FinalityEngine` calculates:
   $$T_{\text{finality}} = t_{\text{certificate\_verified}} - t_{\text{bank\_created}}$$
   The measurement is recorded with provenance `DIRECT` (direct certificate proof).
