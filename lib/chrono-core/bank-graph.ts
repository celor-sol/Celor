import type { CandidateBank, ParentSwitchEvent } from './types';

/**
 * Chrono Bank Graph & Candidate Bank Tracker
 * Models the multi-bank reality of Alpenglow (SIMD-0326 / SIMD-0337).
 * Tracks candidate banks, parent linkages, and processes UpdateParent events.
 */
export class BankGraph {
  // Map of slot -> Map of bankId -> CandidateBank
  private banksBySlot: Map<number, Map<string, CandidateBank>> = new Map();
  private maxRetainedSlots: number = 100;
  private recentParentSwitches: ParentSwitchEvent[] = [];

  /**
   * Register or update a candidate bank.
   */
  public registerBank(bank: CandidateBank): void {
    if (!this.banksBySlot.has(bank.slot)) {
      this.banksBySlot.set(bank.slot, new Map());
    }
    const slotBanks = this.banksBySlot.get(bank.slot)!;
    slotBanks.set(bank.bankId, bank);

    this.pruneOldSlots(bank.slot);
  }

  /**
   * Retrieve all candidate banks for a given slot.
   */
  public getBanksForSlot(slot: number): CandidateBank[] {
    const slotBanks = this.banksBySlot.get(slot);
    if (!slotBanks) return [];
    return Array.from(slotBanks.values()).sort((a, b) => a.bankId.localeCompare(b.bankId));
  }

  /**
   * Processes an UpdateParent signal (SIMD-0337 fast leader handover).
   * Marks the abandoned candidate bank as ABANDONED and promotes the new parent/target.
   */
  public handleUpdateParent(
    slot: number,
    abandonedBankId: string,
    canonicalBankId: string,
    reason: string = 'Fast leader handover parent switch',
    newParentBlockhash: string = '',
    source: string = 'alpenglow-stream'
  ): ParentSwitchEvent {
    const event: ParentSwitchEvent = {
      id: `ps-${slot}-${Date.now()}`,
      slot,
      clearedBankId: abandonedBankId,
      replacementBankId: canonicalBankId,
      abandonedBankId,
      canonicalBankId,
      newParentBlockhash: newParentBlockhash || `parent-${slot - 1}-hash`,
      parentSlot: slot > 0 ? slot - 1 : 0,
      timestampMs: Date.now(),
      reason,
      source,
      provenance: 'DIRECT',
    };

    const slotBanks = this.banksBySlot.get(slot);
    if (slotBanks) {
      const abandoned = slotBanks.get(abandonedBankId);
      if (abandoned) {
        abandoned.state = 'ABANDONED';
      }

      const canonical = slotBanks.get(canonicalBankId);
      if (canonical) {
        canonical.state = 'CANONICAL';
      }
    }

    this.recentParentSwitches.unshift(event);
    if (this.recentParentSwitches.length > 50) {
      this.recentParentSwitches.pop();
    }

    return event;
  }

  /**
   * Set canonical bank for a slot once certified.
   */
  public setCanonicalBank(slot: number, bankId: string): void {
    const slotBanks = this.banksBySlot.get(slot);
    if (slotBanks) {
      for (const [id, bank] of slotBanks.entries()) {
        if (id === bankId) {
          bank.state = 'CANONICAL';
        } else if (bank.state === 'OBSERVED') {
          bank.state = 'ABANDONED';
        }
      }
    }
  }

  /**
   * Set finalized bank for a slot once BLS certificate or TowerBFT root is observed.
   */
  public setFinalizedBank(slot: number, bankId: string): void {
    const slotBanks = this.banksBySlot.get(slot);
    if (slotBanks) {
      const bank = slotBanks.get(bankId);
      if (bank) {
        bank.state = 'FINALIZED';
      }
    }
  }

  public getRecentParentSwitches(): ParentSwitchEvent[] {
    return [...this.recentParentSwitches];
  }

  private pruneOldSlots(currentSlot: number): void {
    const minSlot = currentSlot - this.maxRetainedSlots;
    for (const slot of this.banksBySlot.keys()) {
      if (slot < minSlot) {
        this.banksBySlot.delete(slot);
      }
    }
  }
}
