import { ChronoClient } from '@/lib/chrono-client/client';
import type {
  TransactionAutopsyResult,
  ParentSwitchEvent,
  NetworkStatus,
} from '@/lib/chrono-client/types';

/**
 * ChronoService (Phase 3 Legacy Facade)
 *
 * Provides backwards compatibility for any legacy callers while delegating
 * all consensus state and networking to the authoritative Rust Chrono service.
 */
export class ChronoService {
  private static instance: ChronoService | null = null;
  private client: ChronoClient = ChronoClient.getInstance();

  public static getInstance(): ChronoService {
    if (!ChronoService.instance) {
      ChronoService.instance = new ChronoService();
    }
    return ChronoService.instance;
  }

  public async setCluster(_cluster: string): Promise<void> {
    await this.client.fetchSnapshot();
  }

  public start(): void {
    this.client.start();
  }

  public stop(): void {
    this.client.stop();
  }

  public async analyzeTransaction(signature: string): Promise<TransactionAutopsyResult> {
    return this.client.analyzeTransaction(signature);
  }

  public getNetworkStatus(): NetworkStatus {
    return (this.client as any).networkStatus;
  }
}
