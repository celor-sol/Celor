'use client';

import { useState, useEffect } from 'react';
import { Copy, Check, Terminal, Play, Pause, Trash2, Server, Globe, FileCode } from 'lucide-react';
import { ChronoStreamEvent, NetworkStatus } from '@/lib/chrono-core/types';

interface DevelopersSectionProps {
  streamEvents: ChronoStreamEvent[];
  networkStatus?: NetworkStatus;
}

const codeExamples = [
  {
    label: 'HTTP & WebSocket API (v1)',
    code: `# 1. Fetch current authoritative consensus snapshot
curl -s http://127.0.0.1:8900/api/v1/snapshot | jq .

# 2. Connect to local streaming WebSocket
websocat ws://127.0.0.1:8900/api/v1/stream

# Handshake Request (Client -> Celor Server):
{
  "type": "hello",
  "schema_version": 1,
  "client_id": "client-react-01",
  "last_sequence": 0
}

# Handshake Response (Celor Server -> Client):
{
  "type": "welcome",
  "client_id": "client-react-01",
  "current_sequence": 1042,
  "snapshot_sequence": 1042
}`,
  },
  {
    label: 'TypeScript Client',
    code: `import { CelorClient } from '@/lib/celor-client';

// Connect to local authoritative Rust Celor Service
const celor = CelorClient.getInstance();

// Listen to slot progress events (monotonic sequence tagged)
celor.onSlot((progress) => {
  console.log(\`Slot: \${progress.slot} (\${progress.elapsedMs}ms / \${progress.targetDurationMs}ms)\`);
});

// Listen to candidate bank updates and parent handovers
celor.onParentSwitch((event) => {
  console.log(\`UpdateParent: cleared \${event.clearedBankId} -> canonical \${event.replacementBankId}\`);
});

// Query transaction forensic autopsy directly from Celor Core
const autopsy = await celor.analyzeTransaction("5xY...sig");
console.log(autopsy.confirmationStatus, autopsy.evidence);`,
  },
  {
    label: 'Rust Celor Core',
    code: `use celor_server::service::{CelorServer, ServerConfig};
use celor_server::source::RpcWsSource;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize provider-independent source (Solana Testnet)
    let source = Arc::new(RpcWsSource::new(
        "https://api.testnet.solana.com",
        "wss://api.testnet.solana.com",
        "testnet",
    ));

    // 2. Launch high-throughput local Celor Service (default 127.0.0.1:8900)
    let config = ServerConfig::default();
    let server = CelorServer::new(config, source);
    println!("CELOR Service bound to http://127.0.0.1:8900");
    server.run().await
}`,
  },
  {
    label: 'CLI Engine',
    code: `# Launch local Celor Service daemon
celor serve --port 8900 --cluster testnet

# Or connect to local fixture for reproducible Alpenglow telemetry
celor serve --port 8900 --local

# Execute N>=10,000 in-memory pipeline latency benchmark
celor benchmark --iterations 10000`,
  },
];

const features = [
  {
    title: 'Rust Core Source of Truth',
    description: 'All protocol state, bank graphs, and finality tracked strictly in high-performance Rust.',
  },
  {
    title: 'Zero Browser RPC Networking',
    description: 'Frontend consumes normalized local stream. Zero direct browser-to-Solana RPC latency.',
  },
  {
    title: 'Monotonic Sequence Numbering',
    description: 'Every event carries a sequence number for gap detection and instant ring-buffer replay.',
  },
  {
    title: '$0-First Devnet & Local Fixtures',
    description: 'Runs entirely on open Devnet/Testnet or deterministic local Geyser specification fixtures.',
  },
];

export function DevelopersSection({ streamEvents, networkStatus }: DevelopersSectionProps) {
  const [activeTab, setActiveTab] = useState(-1);
  const [copied, setCopied] = useState(false);
  const [isStreaming, setIsStreaming] = useState(true);
  const [viewRawJson, setViewRawJson] = useState(false);
  const [localEvents, setLocalEvents] = useState<ChronoStreamEvent[]>([]);

  useEffect(() => {
    if (isStreaming && streamEvents.length > 0) {
      setLocalEvents(streamEvents.slice(0, 30));
    }
  }, [isStreaming, streamEvents]);

  const handleCopy = () => {
    if (activeTab >= 0) {
      navigator.clipboard.writeText(codeExamples[activeTab].code);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <section id="developers" className="relative py-16 sm:py-24 lg:py-32 border-t border-foreground/10">
      <div className="max-w-[1400px] mx-auto px-4 sm:px-6 lg:px-12">
        <div className="grid lg:grid-cols-2 gap-12 sm:gap-16 lg:gap-24 items-start">
          {/* Left Column */}
          <div>
            <span className="inline-flex items-center gap-3 text-xs sm:text-sm font-mono text-muted-foreground mb-4 sm:mb-6">
              <span className="w-6 sm:w-8 h-px bg-foreground/30" />
              Developer Primitives
            </span>
            <h2 className="text-3xl sm:text-4xl lg:text-6xl font-display tracking-tight mb-4 sm:mb-8">
              Open infrastructure.
              <br />
              <span className="text-muted-foreground">Zero vendor lock-in.</span>
            </h2>
            <p className="text-base sm:text-lg lg:text-xl text-muted-foreground mb-8 sm:mb-12 leading-relaxed">
              CELOR unifies fragmented RPC WebSockets, Yellowstone gRPC, and validator
              local states into a clean, normalized stream.
            </p>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 sm:gap-6">
              {features.map((feature) => (
                <div key={feature.title} className="p-3 sm:p-0 rounded-xl bg-foreground/[0.02] sm:bg-transparent border sm:border-0 border-foreground/5">
                  <h3 className="font-medium text-foreground mb-1 font-sans text-sm sm:text-base">{feature.title}</h3>
                  <p className="text-xs sm:text-sm text-muted-foreground leading-normal">{feature.description}</p>
                </div>
              ))}
            </div>
          </div>

          {/* Right Column: Code Tabs & Live Terminal */}
          <div className="border border-foreground/10 bg-background rounded-2xl overflow-hidden">
            {/* Tabs */}
            <div className="flex items-center border-b border-foreground/10 overflow-x-auto scrollbar-none">
              <button
                type="button"
                onClick={() => setActiveTab(-1)}
                className={`px-3 sm:px-5 py-3 sm:py-4 text-xs font-mono transition-colors flex items-center gap-1.5 sm:gap-2 shrink-0 ${
                  activeTab === -1
                    ? 'text-foreground font-semibold bg-foreground/[0.03]'
                    : 'text-muted-foreground hover:text-foreground'
                }`}
              >
                <Terminal className="w-3.5 h-3.5 text-green-600" />
                Live Terminal
              </button>

              {codeExamples.map((example, idx) => (
                <button
                  key={example.label}
                  type="button"
                  onClick={() => setActiveTab(idx)}
                  className={`px-3 sm:px-5 py-3 sm:py-4 text-xs font-mono transition-colors relative shrink-0 ${
                    activeTab === idx
                      ? 'text-foreground font-medium'
                      : 'text-muted-foreground hover:text-foreground'
                  }`}
                >
                  {example.label}
                  {activeTab === idx && (
                    <span className="absolute bottom-0 left-0 right-0 h-0.5 bg-foreground" />
                  )}
                </button>
              ))}

              <div className="flex-1" />

              {activeTab >= 0 && (
                <button
                  type="button"
                  onClick={handleCopy}
                  className="px-3 sm:px-4 py-3 sm:py-4 text-muted-foreground hover:text-foreground transition-colors shrink-0"
                  aria-label="Copy code"
                >
                  {copied ? (
                    <Check className="w-4 h-4 text-green-600" />
                  ) : (
                    <Copy className="w-4 h-4" />
                  )}
                </button>
              )}
            </div>

            {/* Tab Content */}
            {activeTab === -1 ? (
              // Live Stream Terminal
              <div className="p-6 font-mono text-xs bg-foreground/[0.01]">
                {/* Service Metadata Ribbon */}
                <div className="mb-4 pb-3 border-b border-foreground/10 grid grid-cols-2 sm:grid-cols-4 gap-2 text-[11px]">
                  <div>
                    <span className="text-muted-foreground block text-[10px] uppercase">Service Status</span>
                    <span className="font-semibold text-emerald-600 dark:text-emerald-400 flex items-center gap-1.5">
                      <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
                      {networkStatus?.chronoStatus || (networkStatus?.connected ? 'LIVE' : 'OFFLINE')}
                    </span>
                  </div>
                  <div>
                    <span className="text-muted-foreground block text-[10px] uppercase">Active Source</span>
                    <span className="text-foreground truncate block">{networkStatus?.source || 'rpc-ws'}</span>
                  </div>
                  <div>
                    <span className="text-muted-foreground block text-[10px] uppercase">Monotonic Sequence</span>
                    <span className="text-foreground font-mono">
                      #{networkStatus?.currentSequence ?? localEvents[0]?.sequence ?? '0'}
                    </span>
                  </div>
                  <div>
                    <span className="text-muted-foreground block text-[10px] uppercase">Schema & Clients</span>
                    <span className="text-foreground">
                      v1 ({networkStatus?.serviceTelemetry?.websocketClients ?? 1} peer)
                    </span>
                  </div>
                </div>

                <div className="flex items-center justify-between pb-3 mb-3 border-b border-foreground/5 text-muted-foreground text-[11px]">
                  <div className="flex items-center gap-2">
                    <button
                      type="button"
                      onClick={() => setViewRawJson(false)}
                      className={`px-2 py-0.5 rounded text-[10px] uppercase transition-colors ${
                        !viewRawJson ? 'bg-foreground text-background font-medium' : 'hover:text-foreground'
                      }`}
                    >
                      Stream View
                    </button>
                    <button
                      type="button"
                      onClick={() => setViewRawJson(true)}
                      className={`px-2 py-0.5 rounded text-[10px] uppercase transition-colors ${
                        viewRawJson ? 'bg-foreground text-background font-medium' : 'hover:text-foreground'
                      }`}
                    >
                      Raw JSON
                    </button>
                  </div>
                  <div className="flex items-center gap-3">
                    <button
                      type="button"
                      onClick={() => setIsStreaming(!isStreaming)}
                      className="hover:text-foreground flex items-center gap-1"
                    >
                      {isStreaming ? (
                        <>
                          <Pause className="w-3 h-3" /> Pause
                        </>
                      ) : (
                        <>
                          <Play className="w-3 h-3" /> Resume
                        </>
                      )}
                    </button>
                    <button
                      type="button"
                      onClick={() => setLocalEvents([])}
                      className="hover:text-foreground flex items-center gap-1"
                    >
                      <Trash2 className="w-3 h-3" /> Clear
                    </button>
                  </div>
                </div>

                {viewRawJson ? (
                  <div className="h-[280px] overflow-y-auto p-3 bg-foreground/[0.02] border border-foreground/5 rounded text-[11px] leading-relaxed">
                    <pre className="text-foreground/90">
                      {localEvents.length > 0
                        ? JSON.stringify(localEvents[0], null, 2)
                        : '// Waiting for incoming CelorServiceEvent...'}
                    </pre>
                  </div>
                ) : (
                  <div className="h-[280px] overflow-y-auto space-y-2 pr-2">
                    {localEvents.length > 0 ? (
                      localEvents.map((ev) => (
                        <div
                          key={ev.id}
                          className="py-1.5 px-2 hover:bg-foreground/[0.02] border-b border-foreground/5 last:border-b-0 flex items-start gap-3"
                        >
                          <span className="text-muted-foreground shrink-0">{ev.timestamp}</span>
                          <span
                            className={`px-1.5 py-0.5 rounded text-[10px] font-semibold shrink-0 ${
                              ev.type === 'CANONICAL'
                                ? 'bg-foreground text-background'
                                : ev.type === 'UPDATE_PARENT'
                                ? 'bg-amber-500 text-black'
                                : 'border border-foreground/10 text-foreground'
                            }`}
                          >
                            {ev.type}
                          </span>
                          <span className="text-muted-foreground">slot {ev.slot}</span>
                          <span className="text-foreground/80 truncate flex-1">{ev.details}</span>
                          <span className="text-muted-foreground text-[10px] uppercase">
                            {ev.source}
                          </span>
                        </div>
                      ))
                    ) : (
                      <div className="h-full flex items-center justify-center text-muted-foreground">
                        Listening to cluster stream...
                      </div>
                    )}
                  </div>
                )}

                <div className="mt-4 pt-3 border-t border-foreground/5 flex flex-wrap items-center justify-between text-[11px] text-muted-foreground gap-2">
                  <span className="flex items-center gap-1.5">
                    <Server className="w-3 h-3 text-muted-foreground" />
                    Local Snapshot: <code className="text-foreground">GET /api/v1/snapshot</code>
                  </span>
                  <span className="flex items-center gap-1.5">
                    <Globe className="w-3 h-3 text-muted-foreground" />
                    WebSocket Stream: <code className="text-foreground">WS /api/v1/stream</code>
                  </span>
                </div>
              </div>
            ) : (
              // Code Block Panel
              <div className="p-8 font-mono text-xs bg-foreground/[0.01] min-h-[340px] overflow-x-auto">
                <pre className="text-foreground/85 leading-relaxed">
                  {codeExamples[activeTab].code}
                </pre>
              </div>
            )}
          </div>
        </div>
      </div>
    </section>
  );
}
