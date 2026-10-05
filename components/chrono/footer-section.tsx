'use client';

import Link from 'next/link';
import { ArrowUpRight } from 'lucide-react';
import { AnimatedWave } from '@/components/visuals/animated-wave';
import { CELOR_TOKEN } from '@/lib/token';

const footerLinks = {
  Protocol: [
    {
      name: 'SIMD-0326 (Alpenglow)',
      href: 'https://github.com/solana-foundation/solana-improvement-documents/blob/main/proposals/0326-alpenglow.md',
    },
    {
      name: 'SIMD-0337 (UpdateParent)',
      href: 'https://github.com/solana-foundation/solana-improvement-documents',
    },
    {
      name: 'SIMD-0525 (Slot Times)',
      href: 'https://github.com/solana-foundation/solana-improvement-documents',
    },
    {
      name: 'Protocol Architecture',
      href: '/protocol',
    },
    {
      name: 'Transition Updates & Log',
      href: '/updates',
    },
  ],
  Architecture: [
    { name: 'Consensus Comparison', href: '/network' },
    { name: 'Benchmark Protocol', href: '/benchmarks' },
    { name: 'Candidate Bank Graph', href: '/telemetry' },
    { name: 'Transaction Autopsy', href: '/transaction' },
    { name: 'Developer APIs & Schemas', href: '/developers' },
  ],
  Community: [
    { name: 'Solana Foundation', href: 'https://solana.com' },
    { name: 'Anza Engineering', href: 'https://anza.xyz' },
    { name: 'Agave Validator Repo', href: 'https://github.com/anza-xyz/agave' },
  ],
  Socials: [
    { name: 'Follow on X (Twitter)', href: CELOR_TOKEN.xUrl },
    { name: 'DexScreener', href: CELOR_TOKEN.dexScreenerUrl },
    { name: 'Pump.fun', href: CELOR_TOKEN.pumpFunUrl },
    { name: 'Solscan', href: CELOR_TOKEN.solscanUrl },
  ],
};

export function FooterSection() {
  return (
    <footer className="relative border-t border-foreground/10 bg-background overflow-hidden">
      {/* Animated wave background */}
      <div className="absolute inset-0 h-64 opacity-20 pointer-events-none overflow-hidden">
        <AnimatedWave />
      </div>

      <div className="relative z-10 max-w-[1400px] mx-auto px-4 sm:px-6 lg:px-12">
        {/* Main Footer */}
        <div className="py-12 sm:py-16 lg:py-24">
          <div className="grid grid-cols-2 md:grid-cols-6 gap-8 sm:gap-12 lg:gap-8">
            {/* Brand Column */}
            <div className="col-span-2">
              <Link href="/" className="inline-flex items-center gap-2.5 mb-4 group">
                <img
                  src="/celor-logo.png?v=5"
                  alt="CELOR"
                  className="w-8 h-8 object-contain shrink-0 group-hover:scale-105 transition-transform"
                />
                <span className="text-2xl font-display font-semibold">CELOR</span>
                <span className="text-xs text-muted-foreground font-mono">SOL</span>
              </Link>

              <p className="text-xs font-mono text-foreground font-medium mb-3">
                The Next infrastructure layer for Solana.
              </p>

              <p className="text-muted-foreground leading-relaxed mb-6 sm:mb-8 max-w-xs text-xs sm:text-sm font-sans">
                Provider-independent Solana consensus timing, bank graph tracking, and
                deterministic finality reconciliation infrastructure (SIMD-0326 protocol target: &lt;150ms).
              </p>

              <div className="text-xs font-mono text-muted-foreground">
                Initial Budget: $0 (Devnet/Testnet First)
              </div>
            </div>

            {/* Link Columns */}
            {Object.entries(footerLinks).map(([title, links]) => (
              <div key={title}>
                <h3 className="text-xs sm:text-sm font-semibold sm:font-medium mb-4 sm:mb-6 font-sans tracking-wide">{title}</h3>
                <ul className="space-y-3 sm:space-y-4">
                  {links.map((link) => {
                    const isExternal = link.href.startsWith('http');
                    return (
                      <li key={link.name}>
                        {isExternal ? (
                          <a
                            href={link.href}
                            target="_blank"
                            rel="noreferrer"
                            className="text-xs sm:text-sm text-muted-foreground hover:text-foreground transition-colors inline-flex items-center gap-1 group"
                          >
                            <span>{link.name}</span>
                            <ArrowUpRight className="w-3 h-3 opacity-0 -translate-x-1 group-hover:opacity-100 group-hover:translate-x-0 transition-all" />
                          </a>
                        ) : (
                          <Link
                            href={link.href}
                            className="text-xs sm:text-sm text-muted-foreground hover:text-foreground transition-colors inline-flex items-center gap-1 group"
                          >
                            <span>{link.name}</span>
                          </Link>
                        )}
                      </li>
                    );
                  })}
                </ul>
              </div>
            ))}
          </div>
        </div>

        {/* Bottom Bar */}
        <div className="py-6 sm:py-8 border-t border-foreground/10 flex flex-col md:flex-row items-start md:items-center justify-between gap-3 sm:gap-4 font-mono text-[11px] sm:text-xs text-muted-foreground">
          <p>© 2026 CELOR Infrastructure Project. Open-Source under MIT/Apache 2.0.</p>

          <div className="flex items-center gap-4">
            <span className="flex items-center gap-2">
              <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
              <span>All consensus modules operational</span>
            </span>
          </div>
        </div>
      </div>
    </footer>
  );
}
