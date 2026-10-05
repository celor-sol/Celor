'use client';

import { useState, useEffect } from 'react';
import { useChrono } from '@/hooks/useChrono';
import { Navigation } from '@/components/chrono/navigation';
import { FooterSection } from '@/components/chrono/footer-section';
import { Button } from '@/components/ui/button';
import { ProvenanceBadge } from '@/components/chrono/provenance-badge';
import { Crosshair, Play, Square, Activity, Zap, Shield, AlertTriangle } from 'lucide-react';

export default function SniperPage() {
  const { networkStatus, switchCluster } = useChrono();
  const [rules, setRules] = useState<any[]>([]);
  const [events, setEvents] = useState<any[]>([]);
  const [decisions, setDecisions] = useState<any[]>([]);

  // Fetch data
  useEffect(() => {
    const fetchData = async () => {
      try {
        const rulesRes = await fetch('http://localhost:8900/api/v1/sniper/rules');
        if (rulesRes.ok) {
          const rulesData = await rulesRes.json();
          setRules(rulesData);
        }

        const eventsRes = await fetch('http://localhost:8900/api/v1/sniper/events');
        if (eventsRes.ok) {
          const eventsData = await eventsRes.json();
          setEvents(eventsData.slice(0, 20));
        }

        const decisionsRes = await fetch('http://localhost:8900/api/v1/sniper/decisions');
        if (decisionsRes.ok) {
          const decisionsData = await decisionsRes.json();
          setDecisions(decisionsData.slice(0, 20));
        }
      } catch (err) {
        console.error("Failed to fetch sniper data", err);
      }
    };

    fetchData();
    const interval = setInterval(fetchData, 1000); 

    // Connect WebSocket for live executions (we can keep this around if needed later)
    const ws = new WebSocket('ws://localhost:8900/api/v1/sniper/ws');
    ws.onmessage = (event) => {
      // You can handle executions here if you want to show a toast
    };

    return () => {
      clearInterval(interval);
      ws.close();
    };
  }, []);

  const toggleRule = async (id: string, currentlyArmed: boolean) => {
    try {
      const action = currentlyArmed ? 'pause' : 'arm';
      await fetch(`http://localhost:8900/api/v1/sniper/rules/${id}/${action}`, {
        method: 'POST',
      });
      // state will update on next poll
    } catch (e) {
      console.error(e);
    }
  };

  const [showAddRule, setShowAddRule] = useState(false);
  const [newRule, setNewRule] = useState({
    name: 'New Snipe Rule',
    program: '11111111111111111111111111111111',
    event_type: 'Pre-Execution',
    earliest_state: 'Candidate',
    route: 'QUIC',
    action: 'FIRE',
    cancel_condition: 'Bank Abandoned',
    leader_target: 'Current',
  });

  const submitRule = async () => {
    try {
      const id = `rule-${Math.floor(Math.random() * 10000)}`;
      await fetch('http://localhost:8900/api/v1/sniper/rules', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id,
          ...newRule,
          armed: true,
        }),
      });
      setShowAddRule(false);
    } catch (e) {
      console.error(e);
    }
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
            CELOR PRE-FINALITY
          </span>
          <h1 className="text-[clamp(2.15rem,6.8vw,3.25rem)] sm:text-[clamp(2.5rem,7vw,5.5rem)] font-display leading-[0.95] tracking-tight mb-3 sm:mb-4">
            Sniper Infrastructure.
          </h1>
          <p className="text-base sm:text-lg lg:text-xl text-muted-foreground max-w-3xl font-sans">
            Plug your execution bot into Celor. Access validator-aware state, alpha candidate streams, and deterministic Fire/Wait/Cancel triggers before the network finalizes blocks.
          </p>
        </div>

        <div className="space-y-12 sm:space-y-16">
          
          {/* Rules Configuration */}
          <div className="space-y-6">
            <div className="flex items-center justify-between pb-3 sm:pb-4 border-b border-foreground/10">
              <h2 className="text-base sm:text-lg font-mono tracking-tight flex items-center gap-2 uppercase text-muted-foreground">
                <Shield className="w-4 h-4" />
                Active Trigger Rules
              </h2>
              <Button size="sm" variant="outline" className="font-mono text-xs" onClick={() => setShowAddRule(!showAddRule)}>
                + ADD RULE
              </Button>
            </div>

            {showAddRule && (
              <div className="p-5 border border-foreground/10 rounded-xl bg-foreground/[0.05] grid grid-cols-2 md:grid-cols-4 gap-4">
                <input className="bg-background border border-foreground/10 p-2 text-xs font-mono rounded" placeholder="Name" value={newRule.name} onChange={e => setNewRule({...newRule, name: e.target.value})} />
                <input className="bg-background border border-foreground/10 p-2 text-xs font-mono rounded" placeholder="Program ID" value={newRule.program} onChange={e => setNewRule({...newRule, program: e.target.value})} />
                <select className="bg-background border border-foreground/10 p-2 text-xs font-mono rounded" value={newRule.event_type} onChange={e => setNewRule({...newRule, event_type: e.target.value})}>
                  <option>Pre-Execution</option>
                  <option>Leader Node Active</option>
                  <option>UpdateParent</option>
                </select>
                <select className="bg-background border border-foreground/10 p-2 text-xs font-mono rounded" value={newRule.earliest_state} onChange={e => setNewRule({...newRule, earliest_state: e.target.value})}>
                  <option>Candidate</option>
                  <option>Pre-Execution</option>
                  <option>Confirmed</option>
                </select>
                <Button className="col-span-full font-mono text-xs" onClick={submitRule}>SAVE & ARM</Button>
              </div>
            )}

            <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6">
              {rules.length === 0 ? (
                <div className="col-span-full p-8 border border-foreground/10 rounded-2xl bg-foreground/[0.02] text-center text-muted-foreground text-sm font-mono">
                  No rules configured
                </div>
              ) : (
                rules.map((rule) => (
                  <div key={rule.id} className="p-5 border border-foreground/10 rounded-xl bg-foreground/[0.02] flex flex-col justify-between">
                    <div>
                      <div className="flex items-center justify-between mb-4">
                        <div className="flex flex-col">
                          <span className="font-semibold text-lg">{rule.name}</span>
                          <span className="text-xs text-muted-foreground font-mono mt-1">ID: {rule.id}</span>
                        </div>
                        <Button
                          size="sm"
                          variant={rule.armed ? 'default' : 'outline'}
                          className={`rounded-full h-8 px-4 text-xs font-mono transition-colors ${
                            rule.armed ? 'bg-foreground text-background hover:bg-foreground/90' : 'text-muted-foreground hover:text-foreground'
                          }`}
                          onClick={() => toggleRule(rule.id, rule.armed)}
                        >
                          {rule.armed ? (
                            <><Square className="w-3 h-3 mr-2 fill-current" /> ARMED</>
                          ) : (
                            <><Play className="w-3 h-3 mr-2" /> PAUSED</>
                          )}
                        </Button>
                      </div>

                      <div className="grid grid-cols-2 gap-3 text-xs font-mono mb-2">
                        <div className="flex flex-col bg-background p-2 rounded border border-foreground/5">
                          <span className="text-muted-foreground text-[10px] mb-1">Target Event</span>
                          <span>{rule.event_type}</span>
                        </div>
                        <div className="flex flex-col bg-background p-2 rounded border border-foreground/5">
                          <span className="text-muted-foreground text-[10px] mb-1">Target Program</span>
                          <span className="truncate" title={rule.program}>{rule.program.substring(0, 8)}...</span>
                        </div>
                        <div className="flex flex-col bg-background p-2 rounded border border-foreground/5">
                          <span className="text-muted-foreground text-[10px] mb-1">Earliest State</span>
                          <span>{rule.earliest_state}</span>
                        </div>
                        <div className="flex flex-col bg-background p-2 rounded border border-foreground/5">
                          <span className="text-muted-foreground text-[10px] mb-1">Cancel Condition</span>
                          <span>{rule.cancel_condition}</span>
                        </div>
                      </div>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>

          {/* Feeds Grid */}
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-8 lg:gap-12">
            
            {/* Live Decisions */}
            <div className="space-y-4">
              <div className="flex items-center justify-between pb-3 sm:pb-4 border-b border-foreground/10">
                <h2 className="text-base sm:text-lg font-mono tracking-tight flex items-center gap-2 uppercase text-muted-foreground">
                  <Activity className="w-4 h-4 text-foreground" />
                  Engine Decisions
                </h2>
                <span className="flex items-center gap-2 text-[10px] sm:text-xs font-mono text-background bg-foreground px-2 py-1 rounded">
                  <span className="relative flex h-1.5 w-1.5">
                    <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-background opacity-75"></span>
                    <span className="relative inline-flex rounded-full h-1.5 w-1.5 bg-background"></span>
                  </span>
                  LIVE EXECUTION
                </span>
              </div>

              <div className="border border-foreground/10 rounded-2xl bg-foreground/[0.02] flex flex-col h-[700px]">
                {decisions.length === 0 ? (
                  <div className="flex-1 flex flex-col items-center justify-center p-12 text-center gap-4">
                    <div className="w-8 h-8 border border-foreground/20 border-t-foreground animate-spin rounded-full" />
                    <div className="text-muted-foreground font-mono text-sm uppercase tracking-widest">
                      Monitoring Alpha Stream
                    </div>
                    <div className="text-xs text-muted-foreground/60 font-sans max-w-sm">
                      The execution engine is connected to the validator network. Target execution signals will appear here when triggers are met.
                    </div>
                  </div>
                ) : (
                  <div className="flex-1 overflow-y-auto scrollbar-hide">
                    <div className="divide-y divide-foreground/10">
                    {decisions.map((dec, i) => (
                      <div key={i} className="p-4 sm:p-5 flex flex-col md:flex-row gap-4 hover:bg-foreground/[0.02] transition-colors">
                        
                        {/* Action Badge */}
                        <div className="shrink-0 flex items-start">
                          <div className={`px-4 py-2 rounded border font-mono text-xs sm:text-sm font-bold flex items-center gap-2 ${
                            dec.action === 'Fire' ? 'bg-foreground text-background border-foreground' :
                            dec.action === 'Cancel' ? 'bg-foreground/[0.05] text-foreground border-foreground/30' :
                            dec.action === 'Wait' ? 'bg-background text-foreground border-foreground/20' :
                            'bg-foreground/5 border-foreground/20 text-muted-foreground'
                          }`}>
                            {dec.action === 'Fire' && <Zap className="w-4 h-4" />}
                            {dec.action === 'Wait' && <Activity className="w-4 h-4" />}
                            {dec.action === 'Cancel' && <AlertTriangle className="w-4 h-4" />}
                            {dec.action.toUpperCase()}
                          </div>
                        </div>

                        {/* Details */}
                        <div className="flex-1 min-w-0 flex flex-col gap-2">
                          <div className="flex items-center justify-between gap-4">
                            <span className="font-mono text-xs text-foreground truncate">{dec.event_id}</span>
                            <span className="text-[10px] font-mono text-muted-foreground shrink-0">
                              {new Date(dec.timestamp_ms).toISOString()}
                            </span>
                          </div>
                          
                          <div className="flex flex-wrap gap-2 text-[10px] font-mono mb-2">
                            <span className="px-2 py-1 bg-background rounded border border-foreground/10 text-muted-foreground">Rule: <span className="text-foreground">{dec.rule_id}</span></span>
                            <span className="px-2 py-1 bg-background rounded border border-foreground/10 text-muted-foreground">Slot: <span className="text-foreground">{dec.trigger_event.slot}</span></span>
                            <span className="px-2 py-1 bg-background rounded border border-foreground/10 text-muted-foreground">State: <span className="text-foreground">{dec.trigger_event.state}</span></span>
                          </div>

                          <div className="bg-background border border-foreground/10 rounded p-4 text-xs font-mono">
                            <div className="text-muted-foreground mb-3 border-b border-foreground/5 pb-2 uppercase tracking-wider text-[10px]">Deterministic Engine Reasoning</div>
                            <ul className="space-y-2">
                              {dec.reasons.map((r: string, idx: number) => (
                                <li key={idx} className="flex items-start gap-2 text-foreground">
                                  {r.startsWith('✓') ? (
                                    <span className="text-foreground mt-0.5"><Zap className="w-3 h-3" /></span>
                                  ) : r.startsWith('✕') ? (
                                    <span className="text-foreground mt-0.5"><AlertTriangle className="w-3 h-3" /></span>
                                  ) : (
                                    <span className="text-foreground mt-0.5"><Activity className="w-3 h-3" /></span>
                                  )}
                                  <span>{r.substring(1).trim()}</span>
                                </li>
                              ))}
                            </ul>
                          </div>
                        </div>
                      </div>
                    ))}
                    </div>
                  </div>
                )}
              </div>
            </div>

            {/* Live Event Stream (Alpha Feed) */}
            <div className="space-y-4">
              <div className="flex items-center justify-between pb-3 sm:pb-4 border-b border-foreground/10">
                <h2 className="text-base sm:text-lg font-mono tracking-tight flex items-center gap-2 uppercase text-muted-foreground">
                  <Zap className="w-4 h-4 text-foreground" />
                  Sniper Target Feed (Pre-Finality)
                </h2>
              </div>
              
              <div className="border border-foreground/10 rounded-2xl bg-foreground/[0.02] flex flex-col h-[700px]">
                {networkStatus.source === 'PublicRPC' ? (
                  <div className="flex-1 flex flex-col items-center justify-center p-12 text-center gap-4">
                    <div className="w-8 h-8 flex items-center justify-center rounded-full bg-red-500/20 text-red-500">
                      <AlertTriangle className="w-4 h-4" />
                    </div>
                    <div className="text-muted-foreground font-mono text-sm uppercase tracking-widest text-red-400">
                      CANDIDATE TELEMETRY UNAVAILABLE
                    </div>
                    <div className="text-xs text-muted-foreground/60 font-sans max-w-sm">
                      Public RPC does not provide unfinalized bank/slot telemetry. 
                      Switch to Geyser or Yellowstone to enable real-time sniper feeds.
                    </div>
                  </div>
                ) : events.length === 0 ? (
                  <div className="flex-1 flex flex-col items-center justify-center p-12 text-center gap-4">
                    <div className="w-full max-w-[200px] h-px bg-gradient-to-r from-transparent via-foreground/20 to-transparent relative overflow-hidden">
                      <div className="absolute top-0 left-0 h-full w-full bg-foreground/20 animate-pulse" />
                    </div>
                    <div className="text-muted-foreground font-mono text-sm uppercase tracking-widest">
                      Awaiting Node Propagation
                    </div>
                  </div>
                ) : (
                  <div className="flex-1 overflow-y-auto scrollbar-hide">
                    <table className="w-full text-left border-collapse text-xs font-mono relative">
                      <thead className="sticky top-0 bg-background/80 backdrop-blur-md z-10">
                        <tr className="bg-foreground/[0.03] border-b border-foreground/10">
                          <th className="p-3 text-muted-foreground font-normal">State</th>
                          <th className="p-3 text-muted-foreground font-normal">Event Type</th>
                          <th className="p-3 text-muted-foreground font-normal">Slot</th>
                          <th className="p-3 text-muted-foreground font-normal">ID / Signature</th>
                          <th className="p-3 text-muted-foreground font-normal">Time</th>
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-foreground/5">
                        {events.map((ev, i) => (
                          <tr key={i} className="hover:bg-foreground/[0.04] transition-colors cursor-default">
                            <td className="p-3">
                              <span className={`px-2.5 py-1 rounded-sm text-[10px] font-medium border ${
                                ev.state === 'PRE-EXECUTION' ? 'bg-background border-foreground/30 text-foreground' :
                                ev.state === 'CANDIDATE' ? 'bg-foreground/[0.05] border-foreground/20 text-muted-foreground' :
                                ev.state === 'ABANDONED' ? 'bg-foreground border-foreground text-background' :
                                'bg-transparent border-foreground/10 text-muted-foreground'
                              }`}>
                                {ev.state}
                              </span>
                            </td>
                            <td className="p-3 whitespace-nowrap text-foreground">{ev.event_type}</td>
                            <td className="p-3 text-muted-foreground">{ev.slot}</td>
                            <td className="p-3">
                              <div className="flex flex-col gap-1 max-w-[200px]">
                                <span className="truncate text-foreground" title={ev.event_id}>{ev.event_id}</span>
                                {ev.leader && <span className="text-[9px] text-muted-foreground truncate">Leader: {ev.leader}</span>}
                              </div>
                            </td>
                            <td className="p-3 text-muted-foreground whitespace-nowrap">
                              {new Date(ev.observed_at).toLocaleTimeString()}
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                )}
              </div>
            </div>

          </div>
        </div>
      </div>
      <FooterSection />
    </main>
  );
}
