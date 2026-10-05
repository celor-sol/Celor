import type { FinalityInfo, CertificateType } from './types';

/**
 * Chrono Finality Engine
 * Measures and tracks finality progression: OBSERVED -> CANONICAL -> FINALIZED.
 * Supports Alpenglow BLS fast-path notarization and legacy TowerBFT root tracking.
 */
export class FinalityEngine {
  private finalityBySlot: Map<number, FinalityInfo> = new Map();
  private recentFinalizedSlots: FinalityInfo[] = [];

  public onSlotObserved(slot: number, timestampMs?: number): FinalityInfo {
    const now = timestampMs ?? Date.now();
    const info: FinalityInfo = {
      slot,
      observedAtMs: now,
      canonicalAtMs: null,
      finalizedAtMs: null,
      finalityLatencyMs: null,
      isFastPath: true,
      stakeParticipatedPercent: 0,
      certificateType: 'UNKNOWN',
      status: 'OBSERVED',
      provenance: 'DIRECT',
    };
    this.finalityBySlot.set(slot, info);
    return info;
  }

  public onSlotCanonical(slot: number, timestampMs?: number): FinalityInfo | null {
    const info = this.finalityBySlot.get(slot);
    if (!info) return null;

    const now = timestampMs ?? Date.now();
    info.canonicalAtMs = now;
    info.status = 'CANONICAL';
    return info;
  }

  public onSlotFinalized(
    slot: number,
    certificateType: CertificateType = 'BLS_FAST_PATH_CERT',
    stakePercent: number = 84.5,
    timestampMs?: number
  ): FinalityInfo | null {
    const info = this.finalityBySlot.get(slot);
    if (!info) return null;

    const now = timestampMs ?? Date.now();
    info.finalizedAtMs = now;
    info.finalityLatencyMs = Math.max(0, now - (info.observedAtMs ?? now));
    info.certificateType = certificateType;
    info.isFastPath = certificateType === 'BLS_FAST_PATH_CERT' || stakePercent >= 80;
    info.stakeParticipatedPercent = stakePercent;
    info.status = 'FINALIZED';
    info.provenance = 'DIRECT';

    this.recentFinalizedSlots.unshift({ ...info });
    if (this.recentFinalizedSlots.length > 50) {
      this.recentFinalizedSlots.pop();
    }

    return info;
  }

  public getFinalityForSlot(slot: number): FinalityInfo | null {
    return this.finalityBySlot.get(slot) || null;
  }

  public getLatestFinality(): FinalityInfo | null {
    return this.recentFinalizedSlots[0] || null;
  }

  public getRecentFinalizedSlots(): FinalityInfo[] {
    return [...this.recentFinalizedSlots];
  }
}
