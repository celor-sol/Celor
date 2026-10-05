# CHRONO — UI Reference & Optimus Design System Mapping

> **Source of Truth**: `/Users/prakhargaur/Desktop/optimus-ui`  
> **Aesthetic Target**: Technical Editorial Minimalism — Apple-level restraint, high-precision typography, hairline borders, kinetic ASCII math visualization, zero crypto clichés.

---

## 1. Design System Tokens & Foundations

### 1.1 Color Palette (OKLCH Base)
- **Base Background**: `oklch(1 0 0)` (Pure White `#FFFFFF`)
- **Primary Foreground**: `oklch(0.145 0 0)` (Deep Charcoal / Near-Black `#121212`)
- **Muted Background**: `oklch(0.97 0 0)` (Subtle Off-White `#F8F8F8`)
- **Muted Text**: `oklch(0.556 0 0)` (Balanced Neutral Gray `#71717A`)
- **Hairline Border**: `oklch(0.922 0 0)` (`border-foreground/10` / `#E4E4E7`)
- **Status Accents**:
  - Live / Finalized / Direct: `#16A34A` (Emerald / Green pulse indicator)
  - Derived / Pending: `#D97706` (Amber / Warm gold)
  - Abandoned / Cleared: `#E11D48` (Rose / Subdued crimson)
  - Unavailable: `#71717A` (Muted gray)

### 1.2 Typography Hierarchy
- **Editorial Display**: `Instrument_Serif` (`--font-instrument-serif`) — Used for massive, authoritative headlines (`text-4xl lg:text-7xl font-display tracking-tight leading-[0.9]`).
- **Body & Controls**: `Instrument_Sans` (`--font-instrument`) — Clean, high-legibility sans-serif for descriptions, labels, and button actions.
- **Protocol Telemetry**: `JetBrains_Mono` (`--font-jetbrains`) — Used for slot numbers, blockhashes, validator public keys, microsecond latency metrics, and provenance badges.

### 1.3 Geometry, Borders & Radius
- **Container Max Width**: `max-w-[1400px]` with responsive padding `px-6 lg:px-12`.
- **Card Radius**: `rounded-2xl` for floating panels, `rounded-xl` for inner stat blocks, `rounded-full` for badges, tags, and action buttons.
- **Hairline Grid Lines**: 8 horizontal lines (`top: 12.5% * i`) and 12 vertical lines (`left: 8.33% * i`) with `bg-foreground/10 opacity-30` generating a technical blueprint aesthetic.
- **Kinetic Background Canvas**: `AnimatedSphere` 3D ASCII point cloud rendering rotating characters (`░▒▓█▀▄▌▐│─┤├┴┬╭╮╰╯`) at 60 FPS in canvas.

---

## 2. Optimus Component Reuse & Chrono Mapping

| Optimus Component | Source Path in Optimus | Reused As / Chrono Role | Technical Adaptation |
|---|---|---|---|
| **`Navigation`** | `components/landing/navigation.tsx` | Global Floating Header (`components/chrono/navigation.tsx`) | Retains scroll-responsive morphing (transparent h-20 → floating pill h-14 with backdrop blur). Maps routes to `/`, `/transaction`, `/network`, `/developers`, with live cluster badge & local mode toggle. |
| **`HeroSection`** | `components/landing/hero-section.tsx` | Live Slot Clock & Leader Banner (`components/chrono/hero-section.tsx`) | Replaces marketing copy with real-time Solana slot progression, dynamic countdown, and instant leader handoff state. |
| **`AnimatedSphere`** | `components/landing/animated-sphere.tsx` | Protocol Geometry Canvas (`components/visuals/animated-sphere.tsx`) | 100% reused canvas rendering ASCII glyphs on mathematical sphere. Background texture for hero and live network views. |
| **`MetricsSection` & `AnimatedCounter`** | `components/landing/metrics-section.tsx` | Alpenglow Telemetry & Coverage Grid | Reuses cubic-eased monotonic counter for Slot number, TPS, Validator count, and Alpenglow Coverage Score (27% → 100%). |
| **`DevelopersSection`** | `components/landing/developers-section.tsx` | Developer Studio & Stream Inspector (`components/chrono/developers-section.tsx`) | Reuses code tabs with animated character reveals (`.dev-code-char`, `.dev-code-line`), showcasing Rust Core snippets, TypeScript client, and live normalized JSON events. |
| **`HowItWorksSection`** | `components/landing/how-it-works-section.tsx` | Consensus State Progression (`components/chrono/finality-section.tsx`) | Reuses Roman-numeral stepper (`I: Observed`, `II: Canonical`, `III: Finalized`) with live code window displaying certificate details. |
| **`FeaturesSection`** | `components/landing/features-section.tsx` | Alpenglow Multi-Bank & Forensic Inspector (`components/chrono/bank-graph-section.tsx`) | Reuses hairline card grid (`01`, `02`, `03`, `04`) with SVG wireframe animations to illustrate Candidate Banks, `UpdateParent` Handover, and BLS Fast-Path Notarization. |
| **`FooterSection`** | `components/landing/footer-section.tsx` | Technical Specification Footer (`components/chrono/footer-section.tsx`) | Clean, minimal footer citing Solana SIMDs (SIMD-0326, SIMD-0337, SIMD-0525), Agave commit hashes, and $0-budget provenance. |

---

## 3. Dedicated Chrono Infrastructure Components

In addition to adapting Optimus components, Chrono introduces specialized infrastructure UI modules:

1. **`ProvenanceBadge` (`components/chrono/provenance-badge.tsx`)**:
   - Displays subtle micro-badges: `DIRECT`, `DERIVED`, `ESTIMATED`, `UNAVAILABLE`.
   - Never obfuscates missing fields on public RPC feeds.
2. **`SlotClockGauge` (`components/chrono/slot-clock-gauge.tsx`)**:
   - Monotonic SVG circular & linear gauge showing 0ms to 250ms slot boundary progression in real time without React render thrashing.
3. **`BankGraphViewer` (`components/chrono/bank-graph-viewer.tsx`)**:
   - Interactive tree rendering: `Slot -> Candidate Bank(s) -> Parent -> Canonical Confirmation -> Finality`.
   - Explicitly displays `bank_id` (scoped to provider) and highlights `UpdateParent` cleared banks in rose/abandoned state without deleting historical nodes.
4. **`TransactionAutopsyCard` (`components/chrono/transaction-autopsy-card.tsx`)**:
   - Forensic transaction lookup explaining whether a transaction was confirmed, dropped, or invalidated by an `UpdateParent` leader switch.
   - Includes "Technical Details" JSON toggle.
5. **`CoverageMeter` (`components/chrono/coverage-meter.tsx`)**:
   - Visual telemetry progress bar (27% on Public RPC, 100% on Local Geyser / Yellowstone) with per-field capability indicators.

---

## 4. Architectural Invariant: Core-to-UI Data Flow

```text
RUST CHRONO CORE / SOLANA CLUSTER STREAM
                │
                ▼ (Web3 Connection / IPC / Event Stream)
UI DATA ADAPTER (lib/chrono-core/chrono-service.ts)
                │
                ├── SlotClock (Monotonic tick)
                ├── BankGraph (State DAG)
                ├── LeaderEngine (Schedule lookup)
                ├── FinalityEngine (Latency & BLS certs)
                └── AutopsyEngine (Transaction forensic resolution)
                │
                ▼ (Typed In-Memory EventBus)
REACT HOOK (hooks/useChrono.ts)
                │
                ▼ (Immutable state snapshots)
OPTIMUS DESIGN SYSTEM UI (App Router Routes)
  ├── / (Live Dashboard)
  ├── /transaction (Autopsy)
  ├── /network (Timeline & Capabilities)
  └── /developers (Developer Studio)
```

**CRITICAL RULE**: The frontend components NEVER execute consensus math, fork choice algorithms, or invent placeholder numbers. If a field is not received from Chrono Core, it is displayed as `UNAVAILABLE`.
