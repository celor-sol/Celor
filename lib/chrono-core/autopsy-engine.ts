import { ChronoClient } from '@/lib/chrono-client/client';
import type { TransactionAutopsyResult } from '@/lib/chrono-client/types';

/**
 * AutopsyEngine (Phase 3 Frontend Bridge)
 * Delegates all transaction autopsy queries to the Rust Chrono Service.
 */
export class AutopsyEngine {
  public async analyzeTransaction(signature: string): Promise<TransactionAutopsyResult> {
    return ChronoClient.getInstance().analyzeTransaction(signature);
  }
}
