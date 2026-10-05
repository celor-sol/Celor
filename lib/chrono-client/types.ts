export type FieldProvenance = 'DIRECT' | 'DERIVED' | 'INFERRED' | 'ESTIMATED' | 'UNAVAILABLE';
export type ChronoServiceStatus = 'LIVE' | 'STALE' | 'DISCONNECTED' | 'DEGRADED' | 'OFFLINE' | 'CONNECTING';
export type ClusterId = 'devnet' | 'testnet' | 'mainnet-beta' | 'localnet' | 'local-geyser' | 'local-validator' | string;
export type BankState = 'OBSERVED' | 'CANONICAL' | 'ABANDONED' | 'FINALIZED' | 'DEAD' | 'ROOTED';
export type CertificateType =
  | 'BLS_FAST_PATH_CERT'
  | 'BLS_FALLBACK_CERT'
  | 'TOWER_BFT_ROOT'
  | 'UNKNOWN';

export interface SlotProgress {
  slot: number;
  targetDurationMs?: number;
  targetSlotDurationMs?: number;
  elapsedMs: number;
  remainingMs: number;
  phaseRatio: number; // 0.0 to 1.0
  driftMs?: number;
  isMeasured?: boolean;
  isRealTime?: boolean;
  provenance: FieldProvenance;
}

export type HandoffState =
  | 'SLOT_CONTINUATION'
  | 'LEADER_TRANSITION_PENDING'
  | 'HANDOVER_IMMINENT'
  | 'BUILDING'
  | 'PARENT_READY'
  | 'HANDOVER'
  | 'UNKNOWN';

export interface LeaderInfo {
  currentLeader: string | null;
  nextLeader: string | null;
  upcomingLeaders?: string[];
  leaderEstimatedDurationMs?: number;
  handoffState: HandoffState;
  lookahead?: Array<{ slot: number; leader: string }>;
  provenance: FieldProvenance;
}

export interface CandidateBank {
  bankId: string;
  rawBankId?: number | null;
  slot: number;
  parentBankId?: string | null;
  parentBlockhash?: string | null;
  blockhash: string | null;
  bankHash?: string | null;
  state: BankState;
  txCount: number;
  observedAtMs: number;
  isUpdateParentTarget?: boolean;
  abandonmentReason?: string;
  provenance: FieldProvenance;
}

export interface ParentSwitchEvent {
  id?: string;
  slot: number;
  clearedBankId: string;
  replacementBankId: string;
  abandonedBankId?: string;
  canonicalBankId?: string;
  newParentBlockhash?: string;
  parentSlot?: number;
  timestampMs?: number;
  observedAtMs?: number;
  authoritativeParent?: string;
  reason: string;
  fecSetIndex?: number;
  source: string;
  provenance: FieldProvenance;
}

export interface ProtocolInfo {
  consensusMode: string;
  alpenglowActive: boolean;
  genesisSlot: number | null;
  targetFinalityMs: number;
  observedFinalityMs: number | null;
  protocolVersion: string;
  consensusEngine: string;
  executionStatus: string;
}

export interface FinalityInfo {
  slot: number;
  mode?: string;
  observedAtMs?: number;
  canonicalAtMs?: number | null;
  finalizedAtMs?: number | null;
  latencyMs?: number | null;
  finalityLatencyMs?: number | null;
  consensusFinalityMs?: number | null;
  providerLatencyMs?: number | null;
  observationLatencyMs?: number | null;
  chronoProcessingUs?: number;
  targetFinalityMs?: number;
  isFastPath?: boolean;
  stakeParticipatedPercent?: number;
  stakePercent?: number;
  certificateType?: CertificateType;
  certType?: string;
  status?: 'OBSERVED' | 'CANONICAL' | 'FINALIZED' | 'WAITING';
  provenance: FieldProvenance;
  rawCertBytes?: string;
}

export interface AlpenglowTelemetry {
  coverageScore: number;
  provenance: Record<string, FieldProvenance>;
  bankHash?: string;
  producerTimeNanos?: number;
  producerUserAgent?: string;
  finalCertificate?: {
    type: string;
    stakePercent: number;
    latencyMs: number;
    status: string;
    rawLen: number;
  };
  notarRewardCert?: {
    stakePercent: number;
    status: string;
    rawLen: number;
  };
  skipRewardCert?: {
    status: string;
    rawLen: number;
  };
  deshredSupported: boolean;
}

export interface ServiceTelemetry {
  eventsReceivedTotal: number;
  eventsNormalizedTotal: number;
  eventsDroppedTotal: number;
  sourceLatencyUs: number;
  processingLatencyUs: number;
  websocketClients: number;
}

export interface NetworkStatus {
  cluster: ClusterId;
  source?: string;
  environment?: 'live' | 'local' | 'fixture';
  connected: boolean;
  chronoStatus?: ChronoServiceStatus;
  rpcEndpoint: string;
  targetSlotDurationMs: number;
  currentSlot: number | null;
  liveTps: number | null;
  activeValidators: number | null;
  lastUpdateMs: number;
  coverageScore?: number;
  telemetry: AlpenglowTelemetry;
  serviceTelemetry?: ServiceTelemetry;
  currentSequence?: number;
  dimensions?: Record<string, string>;
  limitations?: string[];
  protocol?: ProtocolInfo;
}

export interface ChronoStreamEvent {
  id: string;
  sequence?: number;
  timestamp: string;
  type: string;
  slot: number;
  bankId?: string;
  blockhash?: string;
  source: string;
  details: string;
  provenance: FieldProvenance;
}

export interface AutopsyEvidenceItem {
  tier: 'OBSERVED' | 'INFERRED' | 'UNKNOWN';
  title: string;
  detail: string;
  provenance: FieldProvenance;
}

export interface AutopsyTimelineStep {
  step: string;
  stage?: number;
  title?: string;
  status: 'CONFIRMED' | 'PENDING' | 'ABANDONED' | 'UNAVAILABLE';
  timestamp_ms?: number | null;
  timestampMs?: number | null;
  detail: string;
  source: string;
  provenance: FieldProvenance;
}

export interface TransactionAutopsyResult {
  signature: string;
  observed?: boolean;
  slot: number | null;
  block_time?: number | null;
  blockTime?: number | null;
  blockhash?: string | null;
  bank_hash?: string | null;
  bankHash?: string | null;
  fee?: number | null;
  compute_units_consumed?: number | null;
  computeUnitsConsumed?: number | null;
  leader?: string | null;
  bankId?: string | null;
  candidateBank?: string | null;
  parent?: string | null;
  parentRelation?: string | null;
  canonical?: boolean;
  finalized?: boolean;
  confirmationStatus?: string;
  latencyMs?: number | null;
  producerTimeNanos?: number | null;
  producerUserAgent?: string | null;
  err?: string | null;
  parentSwitch?: {
    occurred: boolean;
    clearedBankId?: string;
    replacementBankId?: string;
    abandonedBankId?: string;
    canonicalBankId?: string;
    details?: string;
  } | null;
  certificateInfo?: {
    type: string;
    status: string;
    stakePercent?: number;
    finality_latency_ms?: number | null;
  } | null;
  evidence?: AutopsyEvidenceItem[];
  timeline?: AutopsyTimelineStep[];
  inferredConclusions?: string[];
  unknowns?: string[];
  provenance?: Record<string, FieldProvenance>;
  sourceProvenance?: FieldProvenance;
  error?: string;
}

export interface ExecutionStateWire {
  decision: {
    action: any;
    explanation: string;
    evidence: string[];
  };
  freshness: {
    slot_tier: string;
    slot_elapsed_ms: number;
    slot_target_duration_ms: number;
    leader_tier: string;
    current_leader: string | null;
    next_leader: string | null;
    remaining_window_ms: number;
    blockhash_tier: string;
    blockhash: string;
    blockhash_age_ms: number;
    source_tier: string;
    last_event_received_ago_ms: number;
    bank_tier: string;
    bank_id: string | null;
  };
  leader_window_ms: number;
  target_leader: string | null;
  next_leader: string | null;
  quic_route: {
    leader: string | null;
    tpu_quic_port: number;
    prewarmed: boolean;
    connection_state: string;
    fallback_rpc: boolean;
  };
  mainnet_safety_guard: boolean;
  execution_mode: string;
  timestamps_t0_t10_contract: string[];
}

/**
 * Raw wire format for ChronoSnapshot received from GET /api/v1/snapshot
 */
export interface ChronoSnapshotWire {
  schema_version: number;
  sequence: number;
  snapshot_timestamp_ms: number;
  cluster: string;
  source: string;
  environment: string;
  status: string;
  slot: {
    current_slot: number;
    target_duration_ms: number;
    elapsed_ms: number;
    phase_ratio: number;
    provenance: FieldProvenance;
  };
  leader: {
    current_leader: string | null;
    next_leader: string | null;
    handoff_state: string;
    lookahead?: Array<{ slot: number; leader: string }>;
    provenance: FieldProvenance;
  };
  banks: {
    candidate_banks: Array<{
      bank_id: string;
      raw_bank_id: number | null;
      slot: number;
      parent_bank_id: string | null;
      blockhash: string | null;
      bank_hash: string | null;
      state: string;
      tx_count: number;
      observed_at_ms: number;
      provenance: FieldProvenance;
      abandonment_reason?: string;
    }>;
    canonical_bank: any;
    total_banks_tracked: number;
  };
  parent: {
    last_update_parent: {
      slot: number;
      cleared_bank_id: number | null;
      replacement_bank_id: number | null;
      parent_slot: number;
      parent_block_id: string | null;
      reason: string;
      observed_at_ms: number;
      provenance: FieldProvenance;
    } | null;
    total_update_parents: number;
  };
  protocol?: {
    consensus_mode: string;
    alpenglow_active: boolean;
    genesis_slot: number | null;
    target_finality_ms: number;
    observed_finality_ms: number | null;
    protocol_version: string;
    consensus_engine: string;
    execution_status: string;
  };
  finality: {
    mode: string;
    finality_latency_ms: number | null;
    consensus_finality_ms?: number | null;
    observation_latency_ms?: number | null;
    provider_latency_ms?: number | null;
    chrono_processing_us?: number;
    last_finalized_slot: number | null;
    cert_type: string | null;
    stake_percent: number | null;
    provenance: FieldProvenance;
  };
  capabilities: {
    dimensions: Record<string, string>;
    coverage_score: number;
    limitations: string[];
  };
  network: {
    connected: boolean;
    rpc_endpoint: string;
    live_tps: number | null;
    active_validators: number | null;
    last_update_ms: number;
  };
  telemetry: {
    events_received_total: number;
    events_normalized_total: number;
    events_dropped_total: number;
    source_latency_us: number;
    processing_latency_us: number;
    websocket_clients: number;
  };
  recent_events: Array<{
    schema_version: number;
    sequence: number;
    event_id: number;
    cluster: string;
    source: string;
    environment: string;
    observed_at_ms: number;
    received_at_ms: number;
    slot: number;
    bank_id: number | null;
    blockhash: string | null;
    parent_slot: number | null;
    parent_blockhash: string | null;
    provenance: FieldProvenance;
    event_type: string;
    payload: any;
  }>;
}

export interface FieldAvailabilityDetail {
  field: string;
  protocol_supported: boolean;
  source_supported: boolean;
  currently_observed: boolean;
  provenance: FieldProvenance;
  source_level: string;
  alternate_source: string | null;
  reason_unavailable: string | null;
}

export interface TelemetryCapabilityMatrixWire {
  current_source: string;
  telemetry_level: string;
  observer: {
    observing_validator: string | null;
    producing_validator: string | null;
    transport_type: string;
    cluster_environment: string;
  };
  fields: FieldAvailabilityDetail[];
  core_coverage_percent: number;
  extended_coverage_percent: number;
}

export interface TelemetryStatusWire {
  telemetry_level: string;
  observer: {
    observing_validator: string | null;
    producing_validator: string | null;
    transport_type: string;
    cluster_environment: string;
  } | null;
  counters: {
    events_received_total: number;
    bank_events: number;
    block_footer_events: number;
    update_parent_events: number;
    certificate_events: number;
    deshred_events: number;
    entry_events: number;
    slot_events: number;
    events_dropped_total: number;
    events_parse_failed: number;
    events_unknown: number;
    events_out_of_order: number;
  };
  last_producer_timing: {
    slot: number;
    producer_time_nanos: number;
    chrono_received_at_nanos: number;
    interval_nanos: number;
    interval_ms: number;
    recorded_at_ms: number;
  } | null;
  update_parents_tracked: number;
  recent_update_parents: Array<{
    slot: number;
    cleared_bank_id: number | null;
    replacement_bank_id: number | null;
    parent_slot: number;
    parent_block_id: string | null;
    reason: string;
    observed_at_ms: number;
    provenance: FieldProvenance;
  }>;
}

export interface CertificateItemWire {
  kind: string;
  slot: number;
  block_id: string | null;
  source: string;
  raw_len: number;
  decode_status: string;
  validation_status: string;
  verification_reason: string;
  participant_count: number | null;
  stake_fraction_estimate: number | null;
  received_at_nanos: number;
}
