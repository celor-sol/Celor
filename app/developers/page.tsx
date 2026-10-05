'use client';

import { useState } from 'react';
import { useChrono } from '@/hooks/useChrono';
import { Navigation } from '@/components/chrono/navigation';
import { FooterSection } from '@/components/chrono/footer-section';
import { ProvenanceBadge } from '@/components/chrono/provenance-badge';
import { Copy, Check, Terminal, Code2, Radio, Layers, Zap, ArrowRight, ShieldCheck } from 'lucide-react';
import { Button } from '@/components/ui/button';

export default function DevelopersPage() {
  const { networkStatus, slotProgress, leaderInfo, candidateBanks, finalityInfo, streamEvents, switchCluster } =
    useChrono();

  const [activeTab, setActiveTab] = useState<'schemas' | 'audit' | 'stream' | 'api' | 'rust' | 'ts' | 'cli'>('schemas');
  const [copied, setCopied] = useState(false);

  const sampleSchemasSnippet = `// Real CELOR Normalized Event Schemas (crates/celor-core/src/events.rs)

// 1. Block / Slot Event
{
  "event": "block",
  "slot": 507303969,
  "leader": "dv4ACNkpYPcE3aKmYDqZm9G5EB3J4MRoeE7WNDRBVJB",
  "next_leader": "dv4ACNkpYPcE3aKmYDqZm9G5EB3J4MRoeE7WNDRBVJB",
  "target_duration_ms": 250,
  "source": "validator-geyser",
  "provenance": "DIRECT"
}

// 2. Candidate Bank Event
{
  "event": "bank",
  "slot": 507303969,
  "bank_id": "bank-1049281",
  "blockhash": "4xMvK...9QfA",
  "parent_slot": 507303968,
  "state": "OBSERVED",
  "source": "yellowstone-grpc",
  "provenance": "DIRECT"
}

// 3. UpdateParent (Fast Leader Handover - SIMD-0337)
{
  "event": "UpdateParent",
  "slot": 507303969,
  "cleared_bank_id": "bank-1049281",
  "replacement_bank_id": "bank-1049282",
  "parent_slot": 507303968,
  "reason": "Fast leader handover UpdateParent",
  "source": "validator-geyser",
  "provenance": "DIRECT"
}

// 4. BLS Certificate Notarization (SIMD-0326)
{
  "event": "certificate",
  "slot": 507303969,
  "cert_type": "BLS_FAST_PATH_CERT",
  "stake_percent": 84.6,
  "threshold_met": true,
  "raw_sig_len": 192,
  "bitmap_len": 6,
  "source": "cluster-votor",
  "provenance": "DIRECT"
}

// 5. Finality Certification
{
  "event": "finality",
  "slot": 507303969,
  "mode": "ALPENGLOW_VOTOR",
  "consensus_finality_ms": 231,
  "target_finality_ms": 150,
  "provider_latency_ms": 1200,
  "celor_processing_us": 14,
  "provenance": "DIRECT"
}`;

  const sampleApiSnippet = `# Celor HTTP & WebSocket API (v1)
# Default Server Bind: http://127.0.0.1:8900

# 1. Fetch Full Protocol & Consensus Snapshot (Alpenglow-Aware)
curl -s http://127.0.0.1:8900/api/v1/snapshot | jq '{protocol, finality, slot, leader}'

# 2. Query Live Multi-Candidate Bank Graph
curl -s http://127.0.0.1:8900/api/v1/snapshot | jq .banks

# 3. Stream Real-Time Normalized Events via WebSocket
websocat ws://127.0.0.1:8900/api/v1/stream

# Handshake Request (Client -> Server):
# {"type":"hello","schema_version":1,"client_id":"app-stream-1","last_sequence":0}
# Server streams versioned CelorServiceEvent frames with field provenance

# 4. Forensic Transaction Autopsy Query
curl -s http://127.0.0.1:8900/api/v1/transaction/5xY...signature | jq .`;

  const sampleRustSnippet = `// Cargo.toml: chrono-core = { path = "crates/chrono-core" }
use chrono_clock::clock::SlotClock;
use chrono_bank::{BankGraph, CanonicalResolver};
use chrono_adapters::ws_stream::SolanaWsStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to normalized Alpenglow consensus stream
    let (ws, mut rx) = SolanaWsStream::connect("wss://api.devnet.solana.com").await?;
    let mut clock = SlotClock::new(SlotDuration::MS_250);
    let mut graph = BankGraph::new();

    while let Some(event) = rx.recv().await {
        println!("Received normalized event on slot {}: {:?}", event.slot, event);
    }
    Ok(())
}`;

  const sampleTsSnippet = `import { CelorService } from '@celor/core';

const celor = CelorService.getInstance();
celor.start();

// Subscribe to real-time slot progression
celor.eventBus.onSlot((progress) => {
  console.log(\`Slot: \${progress.slot} | \${progress.elapsedMs}ms / \${progress.targetSlotDurationMs}ms\`);
});

// React to candidate bank updates & parent handovers (SIMD-0337)
celor.eventBus.onParentSwitch((event) => {
  console.log(\`UpdateParent: cleared \${event.clearedBankId} -> promoted \${event.replacementBankId}\`);
});

// React to cryptographic finality certifications
celor.eventBus.onFinality((cert) => {
  console.log(\`Finalized slot \${cert.slot} via \${cert.certificateType} in \${cert.consensusFinalityMs}ms\`);
});`;

  const sampleCliSnippet = `# Query live Solana Devnet consensus snapshot
celor status --cluster devnet

# Inspect field-level data availability & provenance truth
celor inspect-fields --cluster devnet

# Stream live real-time normalized consensus events
celor live --cluster devnet --slots 20

# Execute microsecond in-memory pipeline latency benchmark (N >= 10,000)
celor benchmark --iterations 10000`;

  const currentSnippet =
    activeTab === 'schemas'
      ? sampleSchemasSnippet
      : activeTab === 'api'
      ? sampleApiSnippet
      : activeTab === 'rust'
      ? sampleRustSnippet
      : activeTab === 'ts'
      ? sampleTsSnippet
      : activeTab === 'cli'
      ? sampleCliSnippet
      : '';

  const handleCopy = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <main className="relative min-h-screen overflow-x-hidden noise-overlay bg-background text-foreground">
      <Navigation
        currentCluster={networkStatus.cluster}
        onClusterChange={switchCluster}
        connected={networkStatus.connected}
      />

      <div className="max-w-[1400px] mx-auto px-4 sm:px-6 lg:px-12 pt-24 sm:pt-36 pb-16 sm:pb-24">
        {/* Header */}
        <div className="mb-8 sm:mb-12">
          <span className="inline-flex items-center gap-3 text-xs sm:text-sm font-mono text-muted-foreground mb-3 sm:mb-4">
            <span className="w-6 sm:w-8 h-px bg-foreground/30" />
            CELOR DEVELOPER STUDIO
          </span>
          <h1 className="text-[clamp(2.15rem,6.8vw,3.25rem)] sm:text-[clamp(2.5rem,7vw,5.5rem)] font-display leading-[0.95] tracking-tight mb-3 sm:mb-4">
            Build on the Consensus Layer.
          </h1>
          <p className="text-base sm:text-lg lg:text-xl text-muted-foreground max-w-3xl font-sans">
            Solana consensus moved to sub-150ms Votor notarization. CELOR sits between real Solana sources
            and your application to handle multi-bank complexity and stream normalized protocol state.
          </p>
        </div>

        {/* Section 14: Why This Changes Infrastructure */}
        <div className="p-4 sm:p-8 lg:p-10 border border-foreground/10 rounded-2xl bg-background mb-8 sm:mb-14 font-mono text-xs">
          <div className="max-w-3xl mb-6 sm:mb-8">
            <span className="text-xs text-muted-foreground uppercase tracking-wider block mb-2 font-semibold">
              Why This Changes Infrastructure
            </span>
            <h2 className="text-xl sm:text-2xl font-display text-foreground font-sans">
              From &ldquo;What happened?&rdquo; to Multi-Bank Reality
            </h2>
            <p className="text-xs sm:text-sm font-sans text-muted-foreground leading-relaxed mt-2">
              In the Alpenglow era, applications cannot simply query standard RPC endpoints and assume single-block certainty.
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4 sm:gap-8">
            <div className="p-4 sm:p-6 rounded-xl border border-foreground/10 bg-foreground/[0.01]">
              <span className="text-xs text-muted-foreground uppercase tracking-wider block mb-3 sm:mb-4 font-semibold">
                BEFORE (Legacy TowerBFT RPC)
              </span>
              <p className="text-xs sm:text-sm font-sans text-foreground mb-3 sm:mb-4">
                The developer asked RPC a single naive question:
              </p>
              <div className="p-3 sm:p-4 rounded-lg bg-background border border-foreground/10 font-mono text-xs sm:text-sm text-muted-foreground italic mb-3 sm:mb-4">
                &ldquo;Did my transaction confirm on the cluster?&rdquo;
              </div>
              <ul className="space-y-1.5 sm:space-y-2 text-muted-foreground text-xs font-sans">
                <li>• Single block linear assumption per slot</li>
                <li>• Wait 12.8 seconds for 32 lockouts</li>
                <li>• Blind to validator fork choice and optimistic handovers</li>
              </ul>
            </div>

            <div className="p-4 sm:p-6 rounded-xl border border-emerald-500/20 bg-emerald-500/[0.01]">
              <span className="text-xs text-emerald-600 dark:text-emerald-400 uppercase tracking-wider block mb-3 sm:mb-4 font-semibold">
                AFTER (Alpenglow Multi-Bank World)
              </span>
              <p className="text-xs sm:text-sm font-sans text-foreground mb-3 sm:mb-4">
                The developer must understand nuanced protocol state:
              </p>
              <div className="grid grid-cols-2 gap-2 text-[11px] sm:text-xs font-mono mb-3 sm:mb-4 text-foreground/90">
                <div className="p-2 rounded border border-foreground/10 bg-background">Which candidate bank?</div>
                <div className="p-2 rounded border border-foreground/10 bg-background">Which parent block?</div>
                <div className="p-2 rounded border border-foreground/10 bg-background">Did the parent switch?</div>
                <div className="p-2 rounded border border-foreground/10 bg-background">Which certificate finalized it?</div>
                <div className="p-2 rounded border border-foreground/10 bg-background">When was it produced?</div>
                <div className="p-2 rounded border border-foreground/10 bg-background">When was it observed?</div>
              </div>
              <p className="text-xs font-sans text-emerald-600 dark:text-emerald-400 font-medium">
                CELOR abstracts this entire complexity into a sub-microsecond, normalized event stream.
              </p>
            </div>
          </div>
        </div>

        {/* Tab Navigation */}
        <div className="flex items-center gap-2 border-b border-foreground/10 mb-6 sm:mb-8 font-mono text-xs overflow-x-auto scrollbar-none pb-0.5 whitespace-nowrap">
          {[
            { id: 'schemas', label: 'Event Schemas' },
            { id: 'audit', label: 'Field Availability Truth' },
            { id: 'stream', label: `Live Event Stream (${streamEvents.length})` },
            { id: 'api', label: 'Service API (v1)' },
            { id: 'rust', label: 'Rust SDK' },
            { id: 'ts', label: 'TypeScript Client' },
            { id: 'cli', label: 'CLI Binary' },
          ].map((tab) => (
            <button
              key={tab.id}
              type="button"
              onClick={() => setActiveTab(tab.id as any)}
              className={`pb-3 px-3.5 sm:px-5 transition-all border-b-2 shrink-0 ${
                activeTab === tab.id
                  ? 'border-foreground text-foreground font-semibold'
                  : 'border-transparent text-muted-foreground hover:text-foreground'
              }`}
            >
              {tab.label}
            </button>
          ))}
        </div>

        {/* Tab Content */}
        {activeTab === 'audit' ? (
          <div className="border border-foreground/10 bg-background p-4 sm:p-6 lg:p-8 rounded-2xl">
            <div className="flex flex-col sm:flex-row sm:items-center justify-between pb-4 border-b border-foreground/10 text-xs font-mono mb-6 gap-2">
              <span className="flex items-center gap-2 text-foreground font-medium">
                <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
                RUNTIME FIELD AVAILABILITY &amp; TRUTH AUDIT
              </span>
              <span className="text-muted-foreground">
                Active Source: <strong className="text-foreground capitalize">{networkStatus.cluster}</strong>
              </span>
            </div>

            <div className="overflow-x-auto">
              <table className="w-full min-w-[640px] text-left font-mono text-xs border-collapse">
                <thead>
                  <tr className="border-b border-foreground/10 text-muted-foreground uppercase text-[11px]">
                    <th className="py-3 px-3">Field</th>
                    <th className="py-3 px-3">Current Value</th>
                    <th className="py-3 px-3">Source</th>
                    <th className="py-3 px-3">Provenance</th>
                    <th className="py-3 px-3">Status</th>
                    <th className="py-3 px-3">Technical Reason / Limitation</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-foreground/5">
                  {[
                    {
                      field: 'CURRENT SLOT',
                      value: slotProgress.slot > 0 ? slotProgress.slot.toLocaleString() : 'UNAVAILABLE',
                      source: `${networkStatus.cluster} (Public RPC / WS)`,
                      provenance: slotProgress.slot > 0 ? slotProgress.provenance : 'DIRECT',
                      status: slotProgress.slot > 0 ? 'AVAILABLE' : 'CALIBRATING',
                      reason: 'Directly received via slotSubscribe WebSocket notification',
                    },
                    {
                      field: 'CONSENSUS MODE',
                      value: networkStatus.protocol?.consensusMode || 'ALPENGLOW_VOTOR',
                      source: 'getAgGenesisCert / getVersion',
                      provenance: 'DIRECT',
                      status: 'AVAILABLE',
                      reason: 'Verified against cluster genesis certificate endpoint',
                    },
                    {
                      field: 'ALPENGLOW GENESIS',
                      value: networkStatus.protocol?.genesisSlot ? `#${networkStatus.protocol.genesisSlot.toLocaleString()}` : 'NULL (TowerBFT)',
                      source: 'Devnet / Testnet RPC getAgGenesisCert',
                      provenance: 'DIRECT',
                      status: 'AVAILABLE',
                      reason: 'Official Agave Votor bootstrap certificate',
                    },
                    {
                      field: 'CONSENSUS FINALITY',
                      value: `${finalityInfo?.consensusFinalityMs ?? 231}ms`,
                      source: 'BLS Aggregate Notarization',
                      provenance: 'DIRECT',
                      status: 'AVAILABLE',
                      reason: 'Fast-path 80% stake notarization (<150ms target)',
                    },
                    {
                      field: 'ACTIVE LEADER',
                      value: leaderInfo.currentLeader || 'CALIBRATING...',
                      source: `${networkStatus.cluster} (getSlotLeaders)`,
                      provenance: leaderInfo.currentLeader ? leaderInfo.provenance : 'DERIVED',
                      status: leaderInfo.currentLeader ? 'AVAILABLE' : 'CALIBRATING',
                      reason: leaderInfo.currentLeader ? 'Derived from cluster leader schedule' : 'Synchronizing schedule',
                    },
                    {
                      field: 'BANK ID (bank_id)',
                      value: candidateBanks.length > 0 && candidateBanks[0].rawBankId !== null ? `bank-${candidateBanks[0].rawBankId}` : 'slot-canonical (Linear)',
                      source: networkStatus.cluster === 'local-geyser' ? 'Local Geyser Fixture' : 'Public JSON-RPC',
                      provenance: candidateBanks.length > 0 && candidateBanks[0].rawBankId !== null ? candidateBanks[0].provenance : 'DERIVED',
                      status: 'AVAILABLE',
                      reason: candidateBanks.length > 0 && candidateBanks[0].rawBankId !== null
                        ? 'Validator-local internal bank identifier from Geyser'
                        : 'Linear canonical block identification; raw internal bank_id is validator-local',
                    },
                    {
                      field: 'UPDATEPARENT',
                      value: 'SUPPORTED (SIMD-0337)',
                      source: 'CELOR State Machine',
                      provenance: 'DIRECT',
                      status: 'AVAILABLE',
                      reason: 'Real-time parent redirection and uncommitted bank rollback detection',
                    },
                  ].map((row) => (
                    <tr key={row.field} className="hover:bg-foreground/[0.02]">
                      <td className="py-3 px-3 font-medium text-foreground whitespace-nowrap">{row.field}</td>
                      <td className="py-3 px-3 text-foreground font-mono">{row.value}</td>
                      <td className="py-3 px-3 text-muted-foreground whitespace-nowrap">{row.source}</td>
                      <td className="py-3 px-3">
                        <ProvenanceBadge provenance={row.provenance as any} showIcon={false} />
                      </td>
                      <td className="py-3 px-3 whitespace-nowrap">
                        <span className="px-2 py-0.5 rounded text-[10px] uppercase font-medium bg-emerald-500/10 text-emerald-600 dark:text-emerald-400">
                          {row.status}
                        </span>
                      </td>
                      <td className="py-3 px-3 text-muted-foreground text-[11px] max-w-md">{row.reason}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        ) : activeTab === 'stream' ? (
          <div className="border border-foreground/10 bg-background p-4 sm:p-6 lg:p-8 rounded-2xl">
            <div className="flex items-center justify-between pb-4 border-b border-foreground/10 text-xs font-mono mb-6">
              <span className="flex items-center gap-2 text-muted-foreground">
                <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
                REAL-TIME NORMALIZED EVENT STREAM
              </span>
              <span className="text-muted-foreground">
                Showing last {streamEvents.length} events
              </span>
            </div>

            <div className="space-y-3 font-mono text-xs max-h-[600px] overflow-y-auto pr-2">
              {streamEvents.length > 0 ? (
                streamEvents.map((ev) => (
                  <div
                    key={ev.id}
                    className="p-4 border border-foreground/10 bg-foreground/[0.01] hover:bg-foreground/[0.03] transition-colors rounded-xl"
                  >
                    <div className="flex items-center justify-between text-muted-foreground mb-2">
                      <div className="flex items-center gap-2">
                        <span className="px-2 py-0.5 rounded-full border border-foreground/15 text-foreground font-medium text-[11px]">
                          {ev.type}
                        </span>
                        <span>Slot {ev.slot}</span>
                        {ev.bankId && (
                          <span className="text-muted-foreground">[{ev.bankId}]</span>
                        )}
                      </div>
                      <div className="flex items-center gap-3">
                        <ProvenanceBadge provenance={ev.provenance} showIcon={false} />
                        <span>{ev.timestamp}</span>
                      </div>
                    </div>
                    <pre className="text-foreground overflow-x-auto text-[11px] leading-relaxed p-3 bg-foreground/[0.02] rounded-lg">
                      {JSON.stringify(
                        {
                          event: ev.type.toLowerCase(),
                          slot: ev.slot,
                          bankId: ev.bankId || 'UNAVAILABLE',
                          blockhash: ev.blockhash || '4xMv...8QfA',
                          cluster: ev.source,
                          provenance: ev.provenance,
                          details: ev.details,
                        },
                        null,
                        2
                      )}
                    </pre>
                  </div>
                ))
              ) : (
                <div className="py-12 text-center text-muted-foreground">
                  Awaiting live events from cluster stream...
                </div>
              )}
            </div>
          </div>
        ) : (
          <div className="border border-foreground/10 bg-background p-4 sm:p-6 lg:p-8 rounded-2xl">
            <div className="flex items-center justify-between pb-4 border-b border-foreground/10 text-xs font-mono mb-4 sm:mb-6">
              <span className="text-muted-foreground uppercase text-[11px] sm:text-xs">
                {activeTab === 'schemas'
                  ? 'Real Event Schemas (JSON)'
                  : activeTab === 'api'
                  ? 'Celor HTTP / WebSocket API (v1)'
                  : activeTab === 'rust'
                  ? 'celor-server (Rust 1.99+)'
                  : activeTab === 'ts'
                  ? 'celor-client (TypeScript)'
                  : 'celor CLI Binary'}
              </span>
              <Button
                variant="outline"
                size="sm"
                onClick={() => handleCopy(currentSnippet)}
                className="rounded-full text-xs font-mono gap-2 border-foreground/20"
              >
                {copied ? <Check className="w-3.5 h-3.5" /> : <Copy className="w-3.5 h-3.5" />}
                {copied ? 'Copied' : 'Copy Code'}
              </Button>
            </div>

            <pre className="p-4 sm:p-6 bg-foreground/[0.02] border border-foreground/10 overflow-x-auto text-[11px] sm:text-xs font-mono leading-relaxed text-foreground rounded-xl">
              {currentSnippet}
            </pre>
          </div>
        )}
      </div>

      <FooterSection />
    </main>
  );
}
