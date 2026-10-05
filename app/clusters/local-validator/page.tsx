'use client';

import Link from 'next/link';
import { Navigation } from '@/components/chrono/navigation';
import { FooterSection } from '@/components/chrono/footer-section';
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
  Cpu,
  Copy,
  Check,
} from 'lucide-react';
import { useState } from 'react';

export default function LocalValidatorClusterPage() {
  const { networkStatus, switchCluster } = useChrono();
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
          <span className="text-foreground">Local Validator</span>
          <span>/</span>
          <span className="text-purple-400 font-bold">GEYSER LEVEL 4</span>
        </div>

        {/* Hero Title */}
        <div className="flex flex-col lg:flex-row lg:items-end justify-between gap-6 pb-6 sm:pb-8 border-b border-foreground/10 mb-8 sm:mb-10">
          <div>
            <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-purple-500/10 border border-purple-500/20 text-[11px] sm:text-xs font-mono text-purple-400 font-semibold mb-3 sm:mb-4">
              <span>$0 BUDGET DEVELOPMENT LAB • FULL VALIDATOR ARENA</span>
            </div>
            <h1 className="text-3xl sm:text-5xl lg:text-6xl font-display font-medium tracking-tight mb-3">
              Local Agave Validator
            </h1>
            <p className="text-sm sm:text-base text-muted-foreground max-w-2xl font-sans leading-relaxed">
              Run a local Agave validator node with the Celor Geyser Plugin. Unlocks Level 4 validator telemetry:
              direct memory-mapped candidate bank structs, pre-execution deshred streams, and nanosecond producer timestamps.
            </p>
          </div>

          <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-2 sm:gap-3 shrink-0 w-full sm:w-auto">
            <Button
              onClick={() => switchCluster('local-validator')}
              variant="default"
              size="sm"
              className="rounded-full text-xs font-mono gap-2 h-10 sm:h-9"
            >
              <Zap className="w-3.5 h-3.5" />
              <span>Connect Local Validator</span>
            </Button>
            <Button
              asChild
              variant="outline"
              size="sm"
              className="rounded-full text-xs font-mono border-foreground/20 hover:bg-foreground/5 gap-2 h-10 sm:h-9"
            >
              <Link href="/telemetry">
                <span>View Full Telemetry</span>
                <ArrowRight className="w-3.5 h-3.5" />
              </Link>
            </Button>
          </div>
        </div>

        {/* Metrics Grid */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-8 sm:mb-12 font-mono">
          <div className="p-4 sm:p-6 rounded-2xl border border-foreground/10 bg-foreground/[0.02]">
            <div className="text-[11px] text-muted-foreground uppercase">Telemetry Level</div>
            <div className="text-xl sm:text-2xl font-bold mt-2 text-purple-400">LEVEL 4</div>
            <div className="text-[11px] text-muted-foreground mt-2">
              Full internal validator hooks
            </div>
          </div>

          <div className="p-4 sm:p-6 rounded-2xl border border-foreground/10 bg-foreground/[0.02]">
            <div className="text-[11px] text-muted-foreground uppercase">Core Telemetry Coverage</div>
            <div className="text-2xl sm:text-3xl font-bold mt-2 text-emerald-500">100%</div>
            <div className="text-[11px] text-muted-foreground mt-2">
              Candidate banks + footers + certs
            </div>
          </div>

          <div className="p-4 sm:p-6 rounded-2xl border border-foreground/10 bg-foreground/[0.02]">
            <div className="text-[11px] text-muted-foreground uppercase">Local Arena Finality</div>
            <div className="text-xl sm:text-2xl font-bold mt-2 text-emerald-500">94 ms</div>
            <div className="text-[11px] text-muted-foreground mt-2">
              Zero network serialization delay
            </div>
          </div>

          <div className="p-4 sm:p-6 rounded-2xl border border-foreground/10 bg-foreground/[0.02]">
            <div className="text-[11px] text-muted-foreground uppercase">Infrastructure Cost</div>
            <div className="text-2xl sm:text-3xl font-bold mt-2 text-foreground">$0.00</div>
            <div className="text-[11px] text-muted-foreground mt-2">
              Runs on commodity hardware
            </div>
          </div>
        </div>

        {/* Specification & CLI */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 sm:gap-8 mb-8 sm:mb-12">
          <div className="lg:col-span-2 p-4 sm:p-8 rounded-2xl border border-foreground/10 bg-foreground/[0.01]">
            <h2 className="text-base sm:text-lg font-display font-medium mb-4">Local Validator Capabilities</h2>
            <div className="divide-y divide-foreground/5 font-mono text-xs">
              <div className="py-2.5 sm:py-3 flex flex-col sm:flex-row justify-between gap-1">
                <span className="text-muted-foreground">Geyser Ingestion Hook:</span>
                <span className="font-bold text-emerald-500">Active (Unix Domain Socket / IPC)</span>
              </div>
              <div className="py-2.5 sm:py-3 flex flex-col sm:flex-row justify-between gap-1">
                <span className="text-muted-foreground">Candidate Bank Graph:</span>
                <span className="text-foreground">Full Arena Visibility (SIMD-0326)</span>
              </div>
              <div className="py-2.5 sm:py-3 flex flex-col sm:flex-row justify-between gap-1">
                <span className="text-muted-foreground">UpdateParent Tracking:</span>
                <span className="text-foreground">Instant State Rollback Invariant Active</span>
              </div>
              <div className="py-2.5 sm:py-3 flex flex-col sm:flex-row justify-between gap-1">
                <span className="text-muted-foreground">Deshred Pre-Execution Stream:</span>
                <span className="text-emerald-500 font-semibold">Supported via Geyser IPC</span>
              </div>
              <div className="py-2.5 sm:py-3 flex flex-col sm:flex-row justify-between gap-1">
                <span className="text-muted-foreground">Local JSON-RPC:</span>
                <span className="text-foreground">http://127.0.0.1:8899</span>
              </div>
            </div>
          </div>

          <div className="space-y-4 sm:space-y-6">
            <div className="p-4 sm:p-6 rounded-2xl border border-foreground/10 bg-foreground/[0.02]">
              <div className="flex items-center justify-between mb-3">
                <span className="text-xs font-mono uppercase tracking-wider text-muted-foreground flex items-center gap-1.5">
                  <Terminal className="w-3.5 h-3.5" />
                  Launch Local Geyser Validator
                </span>
                <button
                  onClick={() => copyCommand('cargo run --bin celor -- serve --cluster local-validator --local')}
                  className="text-xs font-mono text-muted-foreground hover:text-foreground flex items-center gap-1"
                >
                  {copied ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <Copy className="w-3.5 h-3.5" />}
                  <span>{copied ? 'Copied' : 'Copy'}</span>
                </button>
              </div>
              <pre className="p-3 bg-foreground/[0.04] border border-foreground/10 rounded-xl text-xs font-mono overflow-x-auto text-foreground">
                <code>cargo run --bin celor -- serve --cluster local-validator --local</code>
              </pre>
            </div>
          </div>
        </div>

        <div className="flex flex-col sm:flex-row flex-wrap gap-2.5 sm:gap-4 pt-6 border-t border-foreground/10">
          <Button asChild variant="outline" className="font-mono text-xs rounded-xl border-foreground/20 h-10 sm:h-9">
            <Link href="/telemetry">Open Validator Control Room</Link>
          </Button>
          <Button asChild variant="outline" className="font-mono text-xs rounded-xl border-foreground/20 h-10 sm:h-9">
            <Link href="/developers">Geyser Plugin Integration Guide</Link>
          </Button>
          <Button asChild variant="outline" className="font-mono text-xs rounded-xl border-foreground/20 h-10 sm:h-9">
            <Link href="/clusters">All Clusters</Link>
          </Button>
        </div>
      </main>

      <FooterSection />
    </div>
  );
}
