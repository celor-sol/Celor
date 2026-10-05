'use client';

import Link from 'next/link';
import { Navigation } from '@/components/chrono/navigation';
import { FooterSection } from '@/components/chrono/footer-section';
import { ProvenanceBadge } from '@/components/chrono/provenance-badge';
import { Button } from '@/components/ui/button';
import { useChrono } from '@/hooks/useChrono';
import {
  FileText,
  Shield,
  Layers,
  Zap,
  ArrowRight,
  ExternalLink,
  CheckCircle2,
  AlertTriangle,
  GitBranch,
  Clock,
  Cpu,
} from 'lucide-react';

export default function ProtocolPage() {
  const { networkStatus, switchCluster } = useChrono();

  return (
    <div className="min-h-screen bg-background text-foreground font-sans selection:bg-foreground selection:text-background">
      <Navigation
        currentCluster={networkStatus.cluster}
        onClusterChange={switchCluster}
        connected={networkStatus.connected}
      />

      <main className="max-w-[1400px] mx-auto px-4 sm:px-6 lg:px-8 pt-24 sm:pt-32 pb-16 sm:pb-24">
        {/* Header */}
        <div className="pb-6 sm:pb-8 border-b border-foreground/10 mb-8 sm:mb-12">
          <div className="flex items-center gap-2 text-[11px] sm:text-xs font-mono text-muted-foreground uppercase tracking-widest mb-3">
            <span>CELOR</span>
            <span>/</span>
            <span className="text-foreground">Protocol Architecture & Specifications</span>
          </div>
          <h1 className="text-3xl sm:text-5xl lg:text-6xl font-display font-medium tracking-tight mb-4">
            The Alpenglow Consensus Architecture
          </h1>
          <p className="text-base sm:text-lg text-muted-foreground max-w-3xl font-sans leading-relaxed">
            Technical analysis of Solana&apos;s fundamental consensus transition: replacement of TowerBFT with Votor,
            aggregate BLS12-381 certificate notarization, multi-candidate bank execution, and fast leader handovers.
          </p>
        </div>

        {/* Section 1: The Core Invariant - Consensus Changed, Execution Did Not */}
        <div className="p-4 sm:p-8 rounded-2xl sm:rounded-3xl border border-foreground/10 bg-foreground/[0.015] mb-8 sm:mb-12">
          <div className="flex items-center gap-2 text-xs font-mono text-emerald-500 uppercase tracking-widest font-semibold mb-3">
            <CheckCircle2 className="w-4 h-4 shrink-0" />
            <span>The Fundamental Architectural Truth</span>
          </div>
          <h2 className="text-xl sm:text-2xl lg:text-3xl font-display font-medium mb-3">
            Consensus Changed. Execution Did Not.
          </h2>
          <p className="text-xs sm:text-sm text-muted-foreground max-w-3xl font-sans leading-relaxed mb-6">
            Alpenglow is exclusively a consensus replacement. It replaces TowerBFT&apos;s on-chain vote transactions and
            progressive 32-slot lockouts with direct off-chain validator voting and cryptographic certificates.
            The Solana Virtual Machine (SVM), transaction execution, program bytecode, account schemas, and priority fees remain 100% unchanged.
          </p>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 sm:gap-6 font-mono text-xs">
            <div className="p-4 sm:p-5 rounded-xl sm:rounded-2xl border border-emerald-500/20 bg-emerald-500/[0.02]">
              <span className="text-emerald-500 font-bold block mb-3 text-xs sm:text-sm">WHAT CHANGED (CONSENSUS)</span>
              <ul className="space-y-2 text-muted-foreground">
                <li className="flex items-start gap-2">
                  <span className="text-emerald-500 shrink-0">✓</span>
                  <span><strong>Consensus Engine:</strong> TowerBFT replaced by Votor</span>
                </li>
                <li className="flex items-start gap-2">
                  <span className="text-emerald-500 shrink-0">✓</span>
                  <span><strong>Finality Time:</strong> ~12.8s lockouts reduced to ~150ms target (231ms observed)</span>
                </li>
                <li className="flex items-start gap-2">
                  <span className="text-emerald-500 shrink-0">✓</span>
                  <span><strong>Validator Voting:</strong> On-chain tx voting replaced by off-chain aggregate BLS signatures</span>
                </li>
                <li className="flex items-start gap-2">
                  <span className="text-emerald-500 shrink-0">✓</span>
                  <span><strong>Slot Reality:</strong> Multiple candidate banks can compete per slot (SIMD-0326)</span>
                </li>
                <li className="flex items-start gap-2">
                  <span className="text-emerald-500 shrink-0">✓</span>
                  <span><strong>Leader Handover:</strong> Fast parent switches with state rollback (SIMD-0337)</span>
                </li>
              </ul>
            </div>

            <div className="p-4 sm:p-5 rounded-xl sm:rounded-2xl border border-foreground/10 bg-foreground/[0.02]">
              <span className="text-foreground font-bold block mb-3 text-xs sm:text-sm">WHAT IS UNCHANGED (EXECUTION)</span>
              <ul className="space-y-2 text-muted-foreground">
                <li className="flex items-start gap-2">
                  <span className="text-blue-400 shrink-0">—</span>
                  <span><strong>Solana Virtual Machine (SVM):</strong> Bytecode execution, eBPF JIT unchanged</span>
                </li>
                <li className="flex items-start gap-2">
                  <span className="text-blue-400 shrink-0">—</span>
                  <span><strong>On-Chain Programs:</strong> Rust, Anchor, and C smart contracts run identically</span>
                </li>
                <li className="flex items-start gap-2">
                  <span className="text-blue-400 shrink-0">—</span>
                  <span><strong>Transactions & Signatures:</strong> Ed25519 user signatures, fee structures identical</span>
                </li>
                <li className="flex items-start gap-2">
                  <span className="text-blue-400 shrink-0">—</span>
                  <span><strong>Account Data Model:</strong> Account state storage, rent, and owner checks identical</span>
                </li>
                <li className="flex items-start gap-2">
                  <span className="text-blue-400 shrink-0">—</span>
                  <span><strong>Priority Fees:</strong> Compute budget program and priority pricing unchanged</span>
                </li>
              </ul>
            </div>
          </div>
        </div>

        {/* Section 2: Detailed SIMD Specifications */}
        <div className="space-y-6 sm:space-y-8 mb-12 sm:mb-16">
          <div className="flex items-center justify-between border-b border-foreground/10 pb-4">
            <h2 className="text-xl sm:text-2xl font-display font-medium">Solana Improvement Documents (SIMDs)</h2>
            <span className="text-[11px] sm:text-xs font-mono text-muted-foreground">Canonical Specifications</span>
          </div>

          {/* SIMD-0326 */}
          <div className="p-4 sm:p-8 rounded-2xl border border-foreground/10 bg-foreground/[0.01]">
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 sm:gap-4 mb-4">
              <div className="flex items-center gap-3">
                <span className="font-mono text-base sm:text-lg font-bold text-foreground">SIMD-0326</span>
                <span className="text-[11px] sm:text-xs font-mono px-3 py-1 rounded-full border border-emerald-500/30 bg-emerald-500/10 text-emerald-500 font-semibold">
                  ACTIVE ON DEVNET
                </span>
              </div>
              <a
                href="https://github.com/solana-foundation/solana-improvement-documents/blob/main/proposals/0326-alpenglow.md"
                target="_blank"
                rel="noreferrer"
                className="text-xs font-mono text-muted-foreground hover:text-foreground flex items-center gap-1"
              >
                <span>Read Official SIMD-0326</span>
                <ExternalLink className="w-3.5 h-3.5" />
              </a>
            </div>

            <h3 className="text-lg sm:text-xl font-display font-medium mb-2">Alpenglow: Fast Finality Consensus & Votor</h3>
            <p className="text-xs sm:text-sm text-muted-foreground font-sans leading-relaxed mb-4">
              Defines the Votor consensus algorithm replacing TowerBFT. Validators directly sign candidate banks using BLS12-381 keypairs.
              Aggregated signatures form cryptographically verifiable certificates:
            </p>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4 font-mono text-xs">
              <div className="p-3.5 sm:p-4 rounded-xl border border-foreground/5 bg-foreground/[0.02]">
                <strong className="text-foreground block mb-1">Fast Path Notarization (~80% Stake)</strong>
                When &gt;80% of active stake notarizes a candidate bank within the first round, the block achieves instant irreversible finality (~150ms target / 231ms observed).
              </div>
              <div className="p-3.5 sm:p-4 rounded-xl border border-foreground/5 bg-foreground/[0.02]">
                <strong className="text-foreground block mb-1">Fallback Path Finalization (~60% Stake)</strong>
                Under asynchronous or high-latency network partitions, a secondary fallback path notarizes candidate banks at 60% stake threshold to guarantee continuous liveness.
              </div>
            </div>
          </div>

          {/* SIMD-0337 */}
          <div className="p-4 sm:p-8 rounded-2xl border border-foreground/10 bg-foreground/[0.01]">
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 sm:gap-4 mb-4">
              <div className="flex items-center gap-3">
                <span className="font-mono text-base sm:text-lg font-bold text-foreground">SIMD-0337</span>
                <span className="text-[11px] sm:text-xs font-mono px-3 py-1 rounded-full border border-emerald-500/30 bg-emerald-500/10 text-emerald-500 font-semibold">
                  ACTIVE ON DEVNET
                </span>
              </div>
              <a
                href="https://github.com/solana-foundation/solana-improvement-documents"
                target="_blank"
                rel="noreferrer"
                className="text-xs font-mono text-muted-foreground hover:text-foreground flex items-center gap-1"
              >
                <span>Solana SIMD Archive</span>
                <ExternalLink className="w-3.5 h-3.5" />
              </a>
            </div>

            <h3 className="text-lg sm:text-xl font-display font-medium mb-2">Fast Leader Handover & UpdateParent State Invalidation</h3>
            <p className="text-xs sm:text-sm text-muted-foreground font-sans leading-relaxed mb-4">
              Under Alpenglow, incoming leaders do not stall waiting for the preceding leader to seal their block.
              A leader begins producing on speculative state and issues an <code className="text-foreground bg-foreground/10 px-1 py-0.5 rounded">UpdateParent</code> marker if the parent fork switches:
            </p>

            <div className="p-3.5 sm:p-4 rounded-xl border border-amber-500/20 bg-amber-500/[0.02] font-mono text-xs text-muted-foreground">
              <strong className="text-amber-500 block mb-1">CELOR Invariant: Immediate State Invalidation</strong>
              When an <code className="text-foreground">UpdateParent</code> marker is emitted, CELOR immediately marks candidate banks rooted in the abandoned parent as ABANDONED, rolling back all uncommitted speculative transactions without deleting historical lineage.
            </div>
          </div>

          {/* SIMD-0525 */}
          <div className="p-4 sm:p-8 rounded-2xl border border-foreground/10 bg-foreground/[0.01]">
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 sm:gap-4 mb-4">
              <div className="flex items-center gap-3">
                <span className="font-mono text-base sm:text-lg font-bold text-foreground">SIMD-0525</span>
                <span className="text-[11px] sm:text-xs font-mono px-3 py-1 rounded-full border border-blue-500/30 bg-blue-500/10 text-blue-400 font-semibold">
                  PLANNED PROGRESSION
                </span>
              </div>
            </div>

            <h3 className="text-lg sm:text-xl font-display font-medium mb-2">Dynamic Slot Duration Reduction</h3>
            <p className="text-xs sm:text-sm text-muted-foreground font-sans leading-relaxed">
              Progressively compresses slot durations from the historic 400ms target down through 350ms, 300ms, 250ms, and targeting 200ms
              as validator telemetry and Turbine propagation pipelines optimize under Alpenglow.
            </p>
          </div>
        </div>

        {/* Section 3: Why slot != block */}
        <div className="p-4 sm:p-8 rounded-2xl sm:rounded-3xl border border-foreground/10 bg-foreground/[0.02] mb-8 sm:mb-12">
          <h2 className="text-xl sm:text-2xl font-display font-medium mb-3">
            The New Reality: Why Slot ≠ Block
          </h2>
          <p className="text-xs sm:text-sm text-muted-foreground font-sans leading-relaxed mb-6 max-w-3xl">
            In legacy Solana, developers assumed a strict 1-to-1 mapping between a slot and a block.
            Under Alpenglow (SIMD-0326), a slot can contain multiple competing candidate banks generated during leader handovers or fork resolution.
            The Agave validator assigns an internal <code className="text-foreground bg-foreground/10 px-1 py-0.5 rounded">bank_id</code> which is strictly validator-local.
            Cross-cluster reconciliation requires block identity (<code className="text-foreground bg-foreground/10 px-1 py-0.5 rounded">blockhash</code>), which CELOR indexes automatically.
          </p>

          <div className="flex flex-col sm:flex-row flex-wrap gap-3 pt-2">
            <Button asChild variant="default" size="sm" className="font-mono text-xs rounded-xl h-10 sm:h-9">
              <Link href="/telemetry">Inspect Candidate Bank Graph</Link>
            </Button>
            <Button asChild variant="outline" size="sm" className="font-mono text-xs rounded-xl border-foreground/20 h-10 sm:h-9">
              <Link href="/network">View Network Pipeline</Link>
            </Button>
            <Button asChild variant="outline" size="sm" className="font-mono text-xs rounded-xl border-foreground/20 h-10 sm:h-9">
              <Link href="/clusters">Explore Clusters</Link>
            </Button>
          </div>
        </div>
      </main>

      <FooterSection />
    </div>
  );
}
