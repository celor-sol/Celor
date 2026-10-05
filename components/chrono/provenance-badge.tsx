'use client';

import React from 'react';
import type { FieldProvenance } from '@/lib/chrono-core/types';

interface ProvenanceBadgeProps {
  provenance: FieldProvenance;
  className?: string;
  showIcon?: boolean;
}

export function ProvenanceBadge({
  provenance,
  className = '',
  showIcon = true,
}: ProvenanceBadgeProps) {
  const styles: Record<FieldProvenance, { text: string; bg: string; dot: string }> = {
    DIRECT: {
      text: 'text-emerald-700 dark:text-emerald-400 border-emerald-500/30',
      bg: 'bg-emerald-500/5',
      dot: 'bg-emerald-500',
    },
    DERIVED: {
      text: 'text-amber-700 dark:text-amber-400 border-amber-500/30',
      bg: 'bg-amber-500/5',
      dot: 'bg-amber-500',
    },
    INFERRED: {
      text: 'text-purple-700 dark:text-purple-400 border-purple-500/30',
      bg: 'bg-purple-500/5',
      dot: 'bg-purple-500',
    },
    ESTIMATED: {
      text: 'text-sky-700 dark:text-sky-400 border-sky-500/30',
      bg: 'bg-sky-500/5',
      dot: 'bg-sky-500',
    },
    UNAVAILABLE: {
      text: 'text-zinc-500 border-zinc-400/20',
      bg: 'bg-zinc-500/5',
      dot: 'bg-zinc-400',
    },
  };

  const style = styles[provenance] || styles.UNAVAILABLE;

  return (
    <span
      className={`inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full border text-[10px] font-mono uppercase tracking-wider ${style.text} ${style.bg} ${className}`}
      title={`Data Provenance: ${provenance}`}
    >
      {showIcon && <span className={`w-1.5 h-1.5 rounded-full ${style.dot}`} />}
      {provenance}
    </span>
  );
}
