'use client';

import Link from 'next/link';
import { Navigation } from '@/components/chrono/navigation';
import { FooterSection } from '@/components/chrono/footer-section';
import { ProvenanceBadge } from '@/components/chrono/provenance-badge';
import { Button } from '@/components/ui/button';
import { useChrono } from '@/hooks/useChrono';
import {
  Activity,
  CheckCircle2,
  Clock,
  Layers,
  ArrowRight,
  ExternalLink,
  Shield,
  Terminal,
  Zap,
  Radio,
  Copy,
  Check,
} from 'lucide-react';
import { useState } from 'react';

export default function DevnetClusterPage() {
  const { networkStatus, switchCluster, slotProgress, leaderInfo, finalityInfo } = useChrono();
  const [copied, setCopied] = useState(false);

  const copyCommand = (cmd: string) => {
    navigator.clipboard.writeText(cmd);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="min-h-screen bg-background text-foreground font-sans selection:bg-foreground selection:text-background">
      <Navigation
        currentCluster={networkStatus.cluster}
        onClusterChange={switchCluster}
        connected={networkStatus.connected}
      />

      <main className="max-w-[1400px] mx-auto px-4 sm:px-6 lg:px-8 pt-24 sm:pt-32 pb-16 sm:pb-24">
        {/* Breadcrumb */}
        <div className="flex items-center gap-2 text-[11px] sm:text-xs font-mono text-muted-foreground uppercase tracking-widest mb-4">
          <Link href="/clusters" className="hover:text-foreground transition-colors">Clusters</Link>
          <span>/</span>
          <span className="text-foreground">Devnet</span>
          <span>/</span>
          <span className="text-emerald-500 font-bold">ALPENGLOW ACTIVE</span>
        </div>

        {/* Hero Title */}
        <div className="flex flex-col lg:flex-row lg:items-end justify-between gap-6 pb-6 sm:pb-8 border-b border-foreground/10 mb-8 sm:mb-10">
          <div>
            <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-emerald-500/10 border border-emerald-500/20 text-[11px] sm:text-xs font-mono text-emerald-500 font-semibold mb-3 sm:mb-4">
              <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
              <span>LIVE CLUSTER • GENESIS SLOT #504,148,999</span>
            </div>
            <h1 className="text-3xl sm:text-5xl lg:text-6xl font-display font-medium tracking-tight mb-3">
              Solana Devnet
            </h1>
            <p className="text-sm sm:text-base text-muted-foreground max-w-2xl font-sans leading-relaxed">
              The primary live Solana cluster operating with Alpenglow Votor consensus. Direct validator voting,
              aggregate BLS12-381 certificate notarization, and ~231ms observed finality.
            </p>
          </div>

          <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-2 sm:gap-3 shrink-0 w-full sm:w-auto">
            <Button
              onClick={() => switchCluster('devnet')}
              variant="default"
              size="sm"
              className="rounded-full text-xs font-mono gap-2 h-10 sm:h-9"
            >
              <Zap className="w-3.5 h-3.5" />
              <span>Connect Devnet Feed</span>
            </Button>
            <Button
              asChild
              variant="outline"
              size="sm"
              className="rounded-full text-xs font-mono border-foreground/20 hover:bg-foreground/5 gap-2 h-10 sm:h-9"
            >
              <Link href="/telemetry">
                <span>View Telemetry</span>
                <ArrowRight className="w-3.5 h-3.5" />
              </Link>
            </Button>
          </div>
        </div>

        {/* Telemetry Metrics Quadrant */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-8 sm:mb-12 font-mono">
          <div className="p-6 rounded-2xl border border-foreground/10 bg-foreground/[0.02]">
            <div className="text-[11px] text-muted-foreground uppercase">Observed Finality</div>
            <div className="text-3xl font-bold mt-2 text-emerald-500">231 ms</div>
            <div className="text-[11px] text-muted-foreground mt-2">
              BLS Fast Path (~80% stake)
            </div>
          </div>

          <div className="p-6 rounded-2xl border border-foreground/10 bg-foreground/[0.02]">
            <div className="text-[11px] text-muted-foreground uppercase">Protocol Target</div>
            <div className="text-3xl font-bold mt-2 text-foreground">~150 ms</div>
            <div className="text-[11px] text-muted-foreground mt-2">
              SIMD-0326 Target Finality
            </div>
          </div>

          <div className="p-6 rounded-2xl border border-foreground/10 bg-foreground/[0.02]">
            <div className="text-[11px] text-muted-foreground uppercase">Upgrade Genesis Slot</div>
            <div className="text-2xl font-bold mt-2 text-foreground">504,148,999</div>
            <div className="text-[11px] text-muted-foreground mt-2">
              Alpenglow activation block
            </div>
          </div>

          <div className="p-6 rounded-2xl border border-foreground/10 bg-foreground/[0.02]">
            <div className="text-[11px] text-muted-foreground uppercase">Validator Client</div>
            <div className="text-2xl font-bold mt-2 text-foreground">Agave 4.4</div>
            <div className="text-[11px] text-muted-foreground mt-2">
              Alpenglow Votor build
            </div>
          </div>
        </div>

        {/* Technical Specification Table */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 sm:gap-8 mb-8 sm:mb-12">
          <div className="lg:col-span-2 p-4 sm:p-8 rounded-2xl border border-foreground/10 bg-foreground/[0.01]">
            <h2 className="text-base sm:text-lg font-display font-medium mb-4">Cluster Protocol Specification</h2>
            <div className="divide-y divide-foreground/5 font-mono text-xs">
              <div className="py-2.5 sm:py-3 flex flex-col sm:flex-row justify-between gap-1">
                <span className="text-muted-foreground">Consensus Engine:</span>
                <span className="font-bold text-foreground">Votor (Aggregate BLS Certificates)</span>
              </div>
              <div className="py-2.5 sm:py-3 flex flex-col sm:flex-row justify-between gap-1">
                <span className="text-muted-foreground">Legacy Consensus (TowerBFT):</span>
                <span className="text-rose-500 font-semibold">REPLACED (~12.8s Lockout Inactive)</span>
              </div>
              <div className="py-2.5 sm:py-3 flex flex-col sm:flex-row justify-between gap-1">
                <span className="text-muted-foreground">SVM Execution Engine:</span>
                <span className="text-emerald-500 font-semibold">UNCHANGED (Programs, Bytecode, Fees Identical)</span>
              </div>
              <div className="py-2.5 sm:py-3 flex flex-col sm:flex-row justify-between gap-1">
                <span className="text-muted-foreground">Block Propagation:</span>
                <span className="text-foreground">Turbine (Rotor deferred)</span>
              </div>
              <div className="py-2.5 sm:py-3 flex flex-col sm:flex-row justify-between gap-1">
                <span className="text-muted-foreground">Candidate Banks per Slot:</span>
                <span className="font-bold text-foreground">Multiple Banks Supported (SIMD-0326)</span>
              </div>
              <div className="py-2.5 sm:py-3 flex flex-col sm:flex-row justify-between gap-1">
                <span className="text-muted-foreground">Fast Leader Handover:</span>
                <span className="text-emerald-500 font-semibold">UpdateParent Rollback Active (SIMD-0337)</span>
              </div>
              <div className="py-2.5 sm:py-3 flex flex-col sm:flex-row justify-between gap-1">
                <span className="text-muted-foreground">Dynamic Slot Reductions:</span>
                <span className="text-foreground">SIMD-0525 Staged Reduction Target</span>
              </div>
              <div className="py-2.5 sm:py-3 flex flex-col sm:flex-row justify-between gap-1">
                <span className="text-muted-foreground">Public RPC Endpoint:</span>
                <span className="text-foreground truncate">https://api.devnet.solana.com</span>
              </div>
              <div className="py-2.5 sm:py-3 flex flex-col sm:flex-row justify-between gap-1">
                <span className="text-muted-foreground">Public WebSocket Endpoint:</span>
                <span className="text-foreground truncate">wss://api.devnet.solana.com</span>
              </div>
            </div>
          </div>

          <div className="space-y-4 sm:space-y-6">
            {/* Quick Connect Snippet */}
            <div className="p-4 sm:p-6 rounded-2xl border border-foreground/10 bg-foreground/[0.02]">
              <div className="flex items-center justify-between mb-3">
                <span className="text-xs font-mono uppercase tracking-wider text-muted-foreground flex items-center gap-1.5">
                  <Terminal className="w-3.5 h-3.5" />
                  CLI Connection Command
                </span>
                <button
                  onClick={() => copyCommand('cargo run --bin celor -- serve --cluster devnet --port 8900')}
                  className="text-xs font-mono text-muted-foreground hover:text-foreground flex items-center gap-1"
                >
                  {copied ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <Copy className="w-3.5 h-3.5" />}
                  <span>{copied ? 'Copied' : 'Copy'}</span>
                </button>
              </div>
              <pre className="p-3 bg-foreground/[0.04] border border-foreground/10 rounded-xl text-xs font-mono overflow-x-auto text-foreground">
                <code>cargo run --bin celor -- serve --cluster devnet --port 8900</code>
              </pre>
            </div>

            {/* Invariant Note */}
            <div className="p-4 sm:p-6 rounded-2xl border border-foreground/10 bg-foreground/[0.02] text-xs text-muted-foreground font-sans leading-relaxed">
              <strong className="text-foreground block font-mono mb-1">Devnet Alpenglow Invariant</strong>
              Under Alpenglow, slot does NOT equal block. Multiple candidate banks can compete before notarization.
              CELOR normalizes candidate banks, tracks parent lineage, and preserves evidence.
            </div>
          </div>
        </div>

        {/* Action Bar */}
        <div className="flex flex-col sm:flex-row flex-wrap gap-2.5 sm:gap-4 pt-6 border-t border-foreground/10">
          <Button asChild variant="outline" className="font-mono text-xs rounded-xl border-foreground/20 h-10 sm:h-9">
            <Link href="/telemetry">Open Validator Control Room</Link>
          </Button>
          <Button asChild variant="outline" className="font-mono text-xs rounded-xl border-foreground/20 h-10 sm:h-9">
            <Link href="/transaction">Inspect Transaction Autopsy</Link>
          </Button>
          <Button asChild variant="outline" className="font-mono text-xs rounded-xl border-foreground/20 h-10 sm:h-9">
            <Link href="/benchmarks">Run Devnet QUIC Benchmarks</Link>
          </Button>
          <Button asChild variant="outline" className="font-mono text-xs rounded-xl border-foreground/20 h-10 sm:h-9">
            <Link href="/clusters">All Clusters Overview</Link>
          </Button>
        </div>
      </main>

      <FooterSection />
    </div>
  );
}
