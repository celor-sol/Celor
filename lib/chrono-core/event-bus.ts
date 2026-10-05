import type {
  SlotProgress,
  LeaderInfo,
  CandidateBank,
  ParentSwitchEvent,
  FinalityInfo,
  NetworkStatus,
  ChronoStreamEvent,
} from './types';

type EventListener<T> = (data: T) => void;

/**
 * Chrono Typed Event Bus
 * Decouples consensus protocol engines from UI rendering components.
 * Ensures UI only rerenders affected components when relevant events fire.
 */
export class ChronoEventBus {
  private listeners: {
    slot: Set<EventListener<SlotProgress>>;
    leader: Set<EventListener<LeaderInfo>>;
    bank: Set<EventListener<CandidateBank[]>>;
    parentSwitch: Set<EventListener<ParentSwitchEvent>>;
    finality: Set<EventListener<FinalityInfo>>;
    network: Set<EventListener<NetworkStatus>>;
    streamEvent: Set<EventListener<ChronoStreamEvent>>;
  } = {
    slot: new Set(),
    leader: new Set(),
    bank: new Set(),
    parentSwitch: new Set(),
    finality: new Set(),
    network: new Set(),
    streamEvent: new Set(),
  };

  public onSlot(listener: EventListener<SlotProgress>): () => void {
    this.listeners.slot.add(listener);
    return () => this.listeners.slot.delete(listener);
  }

  public onLeader(listener: EventListener<LeaderInfo>): () => void {
    this.listeners.leader.add(listener);
    return () => this.listeners.leader.delete(listener);
  }

  public onBank(listener: EventListener<CandidateBank[]>): () => void {
    this.listeners.bank.add(listener);
    return () => this.listeners.bank.delete(listener);
  }

  public onParentSwitch(listener: EventListener<ParentSwitchEvent>): () => void {
    this.listeners.parentSwitch.add(listener);
    return () => this.listeners.parentSwitch.delete(listener);
  }

  public onFinality(listener: EventListener<FinalityInfo>): () => void {
    this.listeners.finality.add(listener);
    return () => this.listeners.finality.delete(listener);
  }

  public onNetwork(listener: EventListener<NetworkStatus>): () => void {
    this.listeners.network.add(listener);
    return () => this.listeners.network.delete(listener);
  }

  public onStreamEvent(listener: EventListener<ChronoStreamEvent>): () => void {
    this.listeners.streamEvent.add(listener);
    return () => this.listeners.streamEvent.delete(listener);
  }

  public emitSlot(data: SlotProgress): void {
    this.listeners.slot.forEach((fn) => fn(data));
  }

  public emitLeader(data: LeaderInfo): void {
    this.listeners.leader.forEach((fn) => fn(data));
  }

  public emitBank(data: CandidateBank[]): void {
    this.listeners.bank.forEach((fn) => fn(data));
  }

  public emitParentSwitch(data: ParentSwitchEvent): void {
    this.listeners.parentSwitch.forEach((fn) => fn(data));
  }

  public emitFinality(data: FinalityInfo): void {
    this.listeners.finality.forEach((fn) => fn(data));
  }

  public emitNetwork(data: NetworkStatus): void {
    this.listeners.network.forEach((fn) => fn(data));
  }

  public emitStreamEvent(data: ChronoStreamEvent): void {
    this.listeners.streamEvent.forEach((fn) => fn(data));
  }
}
