'use client';

import { useEffect, useState, useRef } from 'react';
import { CandidateBank, ParentSwitchEvent } from '@/lib/chrono-client/types';
import { ArrowRight, ArrowDown, AlertTriangle, CheckCircle2, GitFork, Info, ChevronDown, ChevronUp } from 'lucide-react';
import { ProvenanceBadge } from './provenance-badge';

interface BankGraphSectionProps {
  currentSlot: number;
  candidateBanks: CandidateBank[];
  parentSwitch: ParentSwitchEvent | null;
}

export function BankGraphSection({
  currentSlot,
  candidateBanks,
  parentSwitch,
}: BankGraphSectionProps) {
  const [isVisible, setIsVisible] = useState(false);
  const [showTechnicalDetails, setShowTechnicalDetails] = useState(true);
  const sectionRef = useRef<HTMLElement>(null);

  useEffect(() => {
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) setIsVisible(true);
      },
      { threshold: 0.1 }
    );
    if (sectionRef.current) observer.observe(sectionRef.current);
    return () => observer.disconnect();
  }, []);

  const isLocalGeyser = candidateBanks.some((b) => b.bankHash !== undefined);

  return (
    <section
      id="network"
      ref={sectionRef}
      className="relative py-16 sm:py-24 lg:py-32 overflow-hidden border-t border-foreground/10"
    >
      <div className="max-w-[1400px] mx-auto px-4 sm:px-6 lg:px-12">
        {/* Header */}
        <div className="flex flex-col lg:flex-row lg:items-end lg:justify-between gap-6 sm:gap-8 mb-12 sm:mb-16 lg:mb-20">
          <div>
            <span className="inline-flex items-center gap-3 text-xs sm:text-sm font-mono text-muted-foreground mb-4 sm:mb-6">
              <span className="w-6 sm:w-8 h-px bg-foreground/30" />
              Alpenglow Bank Graph (SIMD-0326 / SIMD-0337)
            </span>
            <h2
              className={`text-3xl sm:text-4xl lg:text-6xl font-display tracking-tight transition-all duration-700 ${
                isVisible ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-4'
              }`}
            >
              Candidate banks.
              <br />
              <span className="text-muted-foreground">Slots are no longer single blocks.</span>
            </h2>
          </div>

          <div className="flex flex-col items-start lg:items-end gap-3 text-xs sm:text-sm font-mono text-muted-foreground max-w-md">
            <span>
              Under fast leader handover, leaders initiate blocks optimistically. Multiple
              banks compete per slot until one is notarized by consensus.
            </span>
            <div className="flex items-center gap-2 text-[11px] sm:text-xs text-muted-foreground border border-foreground/10 px-3 py-1.5 rounded-full bg-foreground/[0.02]">
              <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse shrink-0" />
              <span>SIMD-0337 UpdateParent live stream monitor active</span>
            </div>
          </div>
        </div>

        {/* Plain-English Explanation: "MULTIPLE VERSIONS CAN EXIST" */}
        <div className="mb-8 sm:mb-12 p-4 sm:p-6 lg:p-8 border border-foreground/10 rounded-2xl bg-background font-sans">
          <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-4 sm:gap-6 mb-6 sm:mb-8 pb-6 border-b border-foreground/10">
            <div>
              <span className="text-[11px] sm:text-xs font-mono text-muted-foreground uppercase tracking-wider block mb-2">
                Consensus Reality in Normal English
              </span>
              <h3 className="text-xl sm:text-2xl lg:text-3xl font-display text-foreground">
                Multiple versions of a slot can exist simultaneously.
              </h3>
            </div>
            <div className="text-xs sm:text-sm font-mono text-muted-foreground max-w-md">
              <span className="text-foreground font-semibold">Old idea:</span> &ldquo;One slot = one block.&rdquo;
              <br />
              <span className="text-emerald-600 dark:text-emerald-400 font-semibold">New Alpenglow reality:</span> &ldquo;More than one version of a slot can exist while validators decide which one continues.&rdquo;
            </div>
          </div>

          {/* Visual: Version A vs Version B */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 sm:gap-6 items-center font-mono text-xs">
            <div className="p-4 sm:p-5 rounded-xl border border-rose-500/30 bg-rose-500/[0.03]">
              <div className="flex items-center justify-between mb-2">
                <span className="font-semibold text-rose-600 dark:text-rose-400">VERSION A (Candidate)</span>
                <span className="text-[10px] px-2 py-0.5 rounded border border-rose-500/40 text-rose-600 dark:text-rose-400">DROPPED</span>
              </div>
              <p className="text-muted-foreground text-xs font-sans mb-3">
                Built optimistically on an unfinalized predecessor. When validators switch parents via UpdateParent, Version A is abandoned.
              </p>
              <div className="text-[11px] text-muted-foreground space-y-1">
                <div>State: <span className="text-rose-600 dark:text-rose-400">ABANDONED</span></div>
                <div>Uncommitted txs: <span className="text-foreground">State purged &amp; rolled back</span></div>
              </div>
            </div>

            <div className="flex flex-col items-center justify-center text-center p-2 text-muted-foreground">
              <span className="text-xs font-mono mb-1 font-semibold text-foreground">VALIDATOR FORK CHOICE</span>
              <ArrowRight className="w-5 h-5 hidden md:block text-foreground/40 my-1" />
              <ArrowDown className="w-5 h-5 md:hidden text-foreground/40 my-1" />
              <span className="text-[11px] text-muted-foreground">Fast Leader Handover (SIMD-0337)</span>
            </div>

            <div className="p-4 sm:p-5 rounded-xl border border-emerald-500/30 bg-emerald-500/[0.03]">
              <div className="flex items-center justify-between mb-2">
                <span className="font-semibold text-emerald-600 dark:text-emerald-400">VERSION B (Canonical)</span>
                <span className="text-[10px] px-2 py-0.5 rounded border border-emerald-500/40 text-emerald-600 dark:text-emerald-400 font-semibold">CONTINUES</span>
              </div>
              <p className="text-muted-foreground text-xs font-sans mb-3">
                Gathers 80% stake notarization certificate. Promoted to canonical head of the chain.
              </p>
              <div className="text-[11px] text-muted-foreground space-y-1">
                <div>State: <span className="text-emerald-600 dark:text-emerald-400 font-semibold">CANONICAL / NOTARIZED</span></div>
                <div>Consensus: <span className="text-foreground">Next leader builds from Version B</span></div>
              </div>
            </div>
          </div>

          <div className="mt-6 pt-4 border-t border-foreground/10 text-xs font-mono text-muted-foreground flex flex-wrap gap-3 sm:gap-4">
            <span className="font-semibold text-foreground">UNDER THE HOOD:</span>
            <span>bank_id (validator-local node state)</span>
            <span>•</span>
            <span>parent lineage</span>
            <span>•</span>
            <span>UpdateParent marker</span>
            <span>•</span>
            <span>blockhash canonical identity</span>
          </div>
        </div>

        {/* Real Parent Switch Alert Card (Only displays when event occurs) */}
        {parentSwitch ? (
          <div className="mb-8 sm:mb-12 border border-rose-500/30 p-4 sm:p-6 lg:p-8 bg-rose-500/[0.02] rounded-xl transition-all duration-500">
            <div className="flex items-center justify-between gap-3 text-xs font-mono uppercase tracking-wider text-rose-600 dark:text-rose-400 mb-4">
              <div className="flex items-center gap-2">
                <AlertTriangle className="w-4 h-4" />
                <span>PARENT CHANGED — FAST LEADER HANDOVER (SIMD-0337)</span>
              </div>
              <ProvenanceBadge provenance={parentSwitch.provenance} />
            </div>

            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 sm:gap-6 py-4">
              {/* Abandoned Bank */}
              <div className="flex-1 p-4 border border-rose-500/20 bg-background rounded-lg">
                <span className="text-xs font-mono text-muted-foreground block mb-1">
                  PREVIOUS / ABANDONED PARENT
                </span>
                <span className="text-lg sm:text-xl font-mono font-medium block text-rose-600 dark:text-rose-400">
                  {parentSwitch.clearedBankId || parentSwitch.abandonedBankId}
                </span>
                <span className="text-xs font-mono text-muted-foreground mt-1 block">
                  State purged • Uncommitted transactions invalidated
                </span>
              </div>

              {/* Transition Indicator */}
              <div className="flex items-center justify-center text-muted-foreground">
                <ArrowRight className="w-6 h-6 rotate-90 md:rotate-0 text-muted-foreground" />
              </div>

              {/* Canonical Bank */}
              <div className="flex-1 p-4 border border-emerald-500/20 bg-background rounded-lg">
                <span className="text-xs font-mono text-emerald-600 dark:text-emerald-400 font-medium block mb-1">
                  NEW CANONICAL PARENT
                </span>
                <span className="text-lg sm:text-xl font-mono font-medium block text-foreground">
                  {parentSwitch.replacementBankId || parentSwitch.canonicalBankId}
                </span>
                <span className="text-xs font-mono text-muted-foreground mt-1 block">
                  Notarized branch • Longest confirmed parent chain
                </span>
              </div>
            </div>

            <div className="mt-4 pt-4 border-t border-foreground/10 grid grid-cols-2 md:grid-cols-4 gap-4 text-xs font-mono text-muted-foreground">
              <div>
                <span className="text-muted-foreground/60 block">Affected Slot</span>
                <span className="text-foreground">{parentSwitch.slot}</span>
              </div>
              <div>
                <span className="text-muted-foreground/60 block">Telemetry Source</span>
                <span className="text-foreground">{parentSwitch.source}</span>
              </div>
              <div>
                <span className="text-muted-foreground/60 block">Reason</span>
                <span className="text-foreground truncate block">{parentSwitch.reason}</span>
              </div>
              <div>
                <span className="text-muted-foreground/60 block">Event Timestamp</span>
                <span className="text-foreground">
                  {new Date(parentSwitch.timestampMs || parentSwitch.observedAtMs || Date.now()).toLocaleTimeString()}
                </span>
              </div>
            </div>
          </div>
        ) : (
          <div className="mb-8 sm:mb-12 p-3 sm:p-4 border border-foreground/10 bg-foreground/[0.01] rounded-lg flex flex-col sm:flex-row sm:items-center justify-between gap-2 text-xs font-mono text-muted-foreground">
            <span className="flex items-center gap-2">
              <CheckCircle2 className="w-3.5 h-3.5 text-emerald-600 dark:text-emerald-400 shrink-0" />
              No active parent switch in current slot window
            </span>
            <span>Monitoring UpdateParent (SIMD-0337)</span>
          </div>
        )}

        {/* Candidate Banks Header & Technical Details Toggle */}
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 mb-4">
          <span className="text-xs font-mono text-muted-foreground uppercase tracking-wider">
            Observed Slot State (Slot #{currentSlot > 0 ? currentSlot.toLocaleString() : '—'})
          </span>
          <button
            onClick={() => setShowTechnicalDetails(!showTechnicalDetails)}
            className="inline-flex items-center gap-1.5 text-xs font-mono text-muted-foreground hover:text-foreground transition-colors"
          >
            <span>{showTechnicalDetails ? 'Collapse Technical Details' : 'View Technical Details'}</span>
            {showTechnicalDetails ? <ChevronUp className="w-3.5 h-3.5" /> : <ChevronDown className="w-3.5 h-3.5" />}
          </button>
        </div>

        {/* Candidate Banks Grid for Current Slot */}
        {showTechnicalDetails && (
          <div className="border border-foreground/10 rounded-xl overflow-hidden mb-6">
            {/* Header Bar */}
            <div className="px-4 sm:px-6 py-3 sm:py-4 border-b border-foreground/10 flex flex-col sm:flex-row sm:items-center justify-between gap-1 font-mono text-xs text-muted-foreground bg-foreground/[0.01]">
              <span>
                SLOT {currentSlot > 0 ? currentSlot.toLocaleString() : '—'} CONSENSUS BRANCH STATE
              </span>
              <span>{candidateBanks.length > 0 ? `${candidateBanks.length} CANDIDATE(S) RECORDED` : 'SYNCHRONIZING CANDIDATE BRANCHES...'}</span>
            </div>

            {/* Bank Cards */}
            {candidateBanks.length > 0 ? (
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 divide-y md:divide-y-0 md:divide-x divide-foreground/10">
                {candidateBanks.map((bank, index) => {
                  const isCanonical = bank.state === 'CANONICAL' || bank.state === 'FINALIZED';
                  const isAbandoned = bank.state === 'ABANDONED';

                  return (
                    <div
                      key={`${bank.slot}-${bank.bankId}-${index}`}
                      className={`p-4 sm:p-6 lg:p-8 transition-colors duration-300 ${
                        isCanonical ? 'bg-foreground/[0.02]' : isAbandoned ? 'bg-rose-500/[0.02]' : 'bg-background'
                      }`}
                    >
                      <div className="flex items-center justify-between mb-4">
                        <div>
                          <span className="text-lg font-mono font-medium block">{bank.bankId}</span>
                          {bank.rawBankId !== undefined && (
                            <span className="text-[10px] font-mono text-muted-foreground">
                              Local Node ID: #{bank.rawBankId}
                            </span>
                          )}
                        </div>
                        <div className="flex items-center gap-2">
                          <span
                            className={`text-xs font-mono px-2.5 py-1 rounded-full border ${
                              isCanonical
                                ? 'bg-foreground text-background border-foreground font-medium'
                                : isAbandoned
                                ? 'border-rose-500/30 text-rose-600 dark:text-rose-400 bg-rose-500/5'
                                : 'border-foreground/10 text-muted-foreground'
                            }`}
                          >
                            {bank.state}
                          </span>
                          <ProvenanceBadge provenance={bank.provenance} />
                        </div>
                      </div>

                      <div className="space-y-3 font-mono text-xs">
                        <div>
                          <span className="text-muted-foreground block mb-0.5">Blockhash</span>
                          <span className="text-foreground truncate block font-mono">
                            {bank.blockhash || 'UNAVAILABLE (Awaiting commitment)'}
                          </span>
                        </div>

                        {bank.bankHash && (
                          <div>
                            <span className="text-muted-foreground block mb-0.5">Bank Hash (SIMD-0326)</span>
                            <span className="text-foreground truncate block font-mono text-[11px]">
                              {bank.bankHash}
                            </span>
                          </div>
                        )}

                        <div>
                          <span className="text-muted-foreground block mb-0.5">Parent Reference</span>
                          <span className="text-foreground truncate block">{bank.parentBankId}</span>
                        </div>

                        <div className="flex justify-between pt-2 border-t border-foreground/5">
                          <span className="text-muted-foreground">Transactions</span>
                          <span className="text-foreground">
                            {bank.txCount > 0 ? `${bank.txCount} txs` : 'UNAVAILABLE (Public RPC)'}
                          </span>
                        </div>

                        <div className="flex justify-between">
                          <span className="text-muted-foreground">Observed Time</span>
                          <span className="text-foreground">
                            {new Date(bank.observedAtMs).toLocaleTimeString()}
                          </span>
                        </div>

                        {bank.abandonmentReason && (
                          <div className="pt-2 border-t border-rose-500/10 text-rose-600 dark:text-rose-400 text-[11px]">
                            {bank.abandonmentReason}
                          </div>
                        )}
                      </div>
                    </div>
                  );
                })}
              </div>
            ) : currentSlot > 0 ? (
              <div className="grid grid-cols-1 md:grid-cols-2 divide-y md:divide-y-0 md:divide-x divide-foreground/10 p-0">
                {/* Candidate Banks: Honestly Unavailable */}
                <div className="p-4 sm:p-6 lg:p-8 bg-foreground/[0.01]">
                  <div className="flex items-center justify-between mb-4">
                    <span className="text-lg font-mono font-medium text-muted-foreground">
                      CANDIDATE BANKS (SIMD-0326)
                    </span>
                    <ProvenanceBadge provenance="UNAVAILABLE" />
                  </div>
                  <div className="text-xs font-mono text-muted-foreground space-y-3">
                    <p>
                      Standard Solana Public JSON-RPC delivers single linear confirmed blocks upon cluster commitment.
                    </p>
                    <p>
                      Validator-local candidate bank branches, optimistic parent lineages, and internal <code className="text-foreground">bank_id</code> are validator runtime structures requiring Yellowstone gRPC or Local Geyser mode.
                    </p>
                    <div className="pt-3 border-t border-foreground/5 flex items-center gap-2 text-foreground/80">
                      <span className="w-1.5 h-1.5 rounded-full bg-amber-500" />
                      <span>Multiple candidate bank branches require validator-level telemetry</span>
                    </div>
                  </div>
                </div>

                {/* Observed Linear Block: Genuinely Direct */}
                <div className="p-4 sm:p-6 lg:p-8 bg-background">
                  <div className="flex items-center justify-between mb-4">
                    <span className="text-lg font-mono font-medium text-foreground">
                      CURRENT OBSERVED BLOCK
                    </span>
                    <ProvenanceBadge provenance="DIRECT" />
                  </div>
                  <div className="space-y-3 font-mono text-xs">
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Observed Slot</span>
                      <span className="text-foreground font-medium">{currentSlot.toLocaleString()}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Parent Reference</span>
                      <span className="text-foreground font-medium">Slot {(currentSlot - 1).toLocaleString()}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Consensus Model</span>
                      <span className="text-foreground font-medium">Canonical Confirmed Fork</span>
                    </div>
                    <div className="pt-3 border-t border-foreground/5 text-[11px] text-muted-foreground">
                      Direct stream observation from cluster. Not Alpenglow multi-bank candidate telemetry.
                    </div>
                  </div>
                </div>
              </div>
            ) : (
              <div className="p-12 text-center text-xs font-mono text-muted-foreground">
                Awaiting slot observation from cluster stream...
              </div>
            )}
          </div>
        )}

        {/* Technical Invariant Footnote */}
        <div className="text-xs font-mono text-muted-foreground flex flex-col sm:flex-row justify-between gap-2">
          <span>* bank_id is validator-local in Agave 4.3+; not globally unique across cluster</span>
          <span>Cross-provider reconciliation standard: (slot, blockhash)</span>
        </div>
      </div>
    </section>
  );
}
