'use client';

import { Navigation } from '@/components/chrono/navigation';
import { FooterSection } from '@/components/chrono/footer-section';
import { CELOR_TOKEN } from '@/lib/token';
import { XIcon } from '@/components/chrono/x-icon';
import { AnimatedWave } from '@/components/visuals/animated-wave';
import { Copy, ArrowUpRight } from 'lucide-react';
import { useChrono } from '@/hooks/useChrono';

export default function ContractPage() {
  const { networkStatus, switchCluster } = useChrono();

  return (
    <main className="relative min-h-screen overflow-x-hidden bg-background text-foreground selection:bg-foreground selection:text-background">
      <Navigation
        currentCluster={networkStatus.cluster}
        onClusterChange={switchCluster}
        connected={networkStatus.connected}
      />

      {/* Hero / Contract Section */}
      <section className="relative pt-32 sm:pt-40 pb-20 sm:pb-32 overflow-hidden border-b border-foreground/10">
        <div className="absolute inset-0 opacity-20 pointer-events-none mix-blend-screen overflow-hidden">
          <AnimatedWave />
        </div>

        <div className="relative z-10 max-w-[1400px] mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex flex-col items-center text-center max-w-4xl mx-auto mb-16">
            <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-foreground/10 bg-foreground/[0.02] mb-6">
              <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse shrink-0" />
              <span className="text-xs font-mono text-muted-foreground uppercase tracking-widest">
                Official Contract
              </span>
            </div>

            <h1 className="text-4xl sm:text-5xl md:text-7xl font-display font-medium tracking-tight leading-[1.1] mb-6">
              The CELOR Token.
            </h1>
            
            <p className="text-lg sm:text-xl text-muted-foreground max-w-2xl font-sans mb-12">
              Powering the provider-independent Solana consensus timing and finality reconciliation infrastructure.
            </p>

            {/* Massive CA Display Box */}
            <div className="w-full relative overflow-hidden rounded-3xl border border-foreground/15 bg-background/50 backdrop-blur-xl shadow-2xl p-6 sm:p-10 group">
              <div className="absolute inset-0 bg-gradient-to-br from-foreground/[0.03] to-transparent pointer-events-none" />
              
              <div className="relative flex flex-col gap-6">
                <div className="flex flex-col items-center sm:items-start text-left w-full gap-2 border-b border-foreground/10 pb-6">
                  <div className="text-xs font-mono text-muted-foreground uppercase tracking-widest">Contract Address (Solana)</div>
                  <div className="font-mono text-xl sm:text-2xl md:text-4xl text-foreground font-bold break-all text-center sm:text-left w-full selection:bg-emerald-500/30 selection:text-emerald-900">
                    {CELOR_TOKEN.ca}
                  </div>
                </div>

                <div className="flex flex-col sm:flex-row items-center justify-between gap-4 w-full pt-2">
                  <div className="flex flex-wrap items-center justify-center sm:justify-start gap-3 w-full">
                    <button
                      onClick={() => {
                        navigator.clipboard.writeText(CELOR_TOKEN.ca);
                      }}
                      className="flex items-center gap-2 px-6 py-3 rounded-full bg-foreground text-background text-sm font-semibold hover:bg-foreground/90 transition-all hover:scale-105 active:scale-95"
                    >
                      <Copy className="w-4 h-4" />
                      <span>Copy Address</span>
                    </button>
                    
                    <a
                      href={CELOR_TOKEN.pumpFunUrl}
                      target="_blank"
                      rel="noreferrer"
                      className="flex items-center gap-2 px-6 py-3 rounded-full border border-foreground/20 text-foreground hover:bg-foreground/5 text-sm font-semibold transition-all"
                    >
                      <span>Buy on Pump.fun</span>
                      <ArrowUpRight className="w-4 h-4" />
                    </a>
                  </div>

                  <div className="flex items-center gap-3 shrink-0">
                    <a
                      href={CELOR_TOKEN.xUrl}
                      target="_blank"
                      rel="noreferrer"
                      className="flex items-center justify-center w-12 h-12 rounded-full border border-foreground/10 text-muted-foreground hover:text-foreground hover:bg-foreground/5 transition-all group-hover:border-foreground/30"
                      title="Follow on X"
                    >
                      <XIcon className="w-5 h-5" />
                    </a>
                  </div>
                </div>
              </div>
            </div>
          </div>
          
          {/* External Ecosystem Links */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 max-w-4xl mx-auto">
             {[
               { name: 'DexScreener', url: CELOR_TOKEN.dexScreenerUrl },
               { name: 'Solscan', url: CELOR_TOKEN.solscanUrl },
               { name: 'Birdeye', url: CELOR_TOKEN.birdeyeUrl },
               { name: 'Photon', url: CELOR_TOKEN.photonUrl }
             ].map((link) => (
                <a
                  key={link.name}
                  href={link.url}
                  target="_blank"
                  rel="noreferrer"
                  className="flex items-center justify-between p-4 rounded-xl border border-foreground/10 bg-foreground/[0.01] hover:bg-foreground/[0.03] transition-colors group"
                >
                  <span className="font-mono text-sm font-medium">{link.name}</span>
                  <ArrowUpRight className="w-4 h-4 text-muted-foreground opacity-50 group-hover:opacity-100 group-hover:translate-x-0.5 group-hover:-translate-y-0.5 transition-all" />
                </a>
             ))}
          </div>

        </div>
      </section>

      <FooterSection />
    </main>
  );
}
