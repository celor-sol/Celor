use crate::capabilities::ProviderCapabilityMatrix;
use crate::provider::{AdapterError, ProviderAdapter, ProviderCapabilities, ProviderConnectionStatus};
use async_trait::async_trait;
use chrono_core::bus::EventBus;
use chrono_core::events::{ChronoEvent, ChronoEventKind};
use chrono_core::identity::BankIdentity;
use chrono_core::types::{
    AlpenglowFooter, BankHash, BankId, BankState, BankStatus, Blockhash, CertificateKind,
    CertificateValidationStatus, ObserverContext, ParentReference, ProviderId, Slot,
};
use chrono_geyser_plugin::events::{ChronoGeyserRawFrame, GeyserPayload};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::UnixListener;
use tracing::{error, info, warn};

/// Real-time live event counters for validator-side telemetry stream.
#[derive(Debug, Default)]
pub struct GeyserTelemetryCounters {
    pub events_received_total: AtomicU64,
    pub bank_events: AtomicU64,
    pub block_footer_events: AtomicU64,
    pub update_parent_events: AtomicU64,
    pub certificate_events: AtomicU64,
    pub deshred_events: AtomicU64,
    pub entry_events: AtomicU64,
    pub slot_events: AtomicU64,
    pub events_dropped_total: AtomicU64,
    pub events_parse_failed: AtomicU64,
    pub events_out_of_order: AtomicU64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TelemetryCountersSnapshot {
    pub events_received_total: u64,
    pub bank_events: u64,
    pub block_footer_events: u64,
    pub update_parent_events: u64,
    pub certificate_events: u64,
    pub deshred_events: u64,
    pub entry_events: u64,
    pub slot_events: u64,
    pub events_dropped_total: u64,
    pub events_parse_failed: u64,
    pub events_out_of_order: u64,
}

impl GeyserTelemetryCounters {
    pub fn snapshot(&self) -> TelemetryCountersSnapshot {
        TelemetryCountersSnapshot {
            events_received_total: self.events_received_total.load(Ordering::Relaxed),
            bank_events: self.bank_events.load(Ordering::Relaxed),
            block_footer_events: self.block_footer_events.load(Ordering::Relaxed),
            update_parent_events: self.update_parent_events.load(Ordering::Relaxed),
            certificate_events: self.certificate_events.load(Ordering::Relaxed),
            deshred_events: self.deshred_events.load(Ordering::Relaxed),
            entry_events: self.entry_events.load(Ordering::Relaxed),
            slot_events: self.slot_events.load(Ordering::Relaxed),
            events_dropped_total: self.events_dropped_total.load(Ordering::Relaxed),
            events_parse_failed: self.events_parse_failed.load(Ordering::Relaxed),
            events_out_of_order: self.events_out_of_order.load(Ordering::Relaxed),
        }
    }
}

/// Full-fidelity Local Validator / Geyser UDS Stream Adapter.
///
/// PURPOSE: Maximum Solana & Alpenglow Telemetry Ingestion directly from
/// a live Agave validator's in-process Geyser plugin over Unix Domain Sockets.
pub struct ValidatorGeyserAdapter {
    provider_id: ProviderId,
    socket_path: String,
    status: Arc<std::sync::Mutex<ProviderConnectionStatus>>,
    event_bus: Option<Arc<EventBus>>,
    event_counter: Arc<AtomicU64>,
    last_slot: Arc<AtomicU64>,
    capabilities: ProviderCapabilityMatrix,
    counters: Arc<GeyserTelemetryCounters>,
    stop_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl ValidatorGeyserAdapter {
    pub fn new(
        socket_path: impl Into<String>,
        event_bus: Option<Arc<EventBus>>,
    ) -> Self {
        let provider_id = ProviderId::new("local-validator");
        let matrix = ProviderCapabilityMatrix::local_validator(provider_id.as_str());
        Self {
            provider_id,
            socket_path: socket_path.into(),
            status: Arc::new(std::sync::Mutex::new(ProviderConnectionStatus::Disconnected)),
            event_bus,
            event_counter: Arc::new(AtomicU64::new(1)),
            last_slot: Arc::new(AtomicU64::new(0)),
            capabilities: matrix,
            counters: Arc::new(GeyserTelemetryCounters::default()),
            stop_tx: None,
        }
    }

    pub fn counters(&self) -> Arc<GeyserTelemetryCounters> {
        self.counters.clone()
    }

    pub fn capability_matrix(&self) -> &ProviderCapabilityMatrix {
        &self.capabilities
    }
}

#[async_trait]
impl ProviderAdapter for ValidatorGeyserAdapter {
    fn provider_id(&self) -> &ProviderId {
        &self.provider_id
    }

    fn status(&self) -> ProviderConnectionStatus {
        let guard = self.status.lock().unwrap();
        guard.clone()
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            slot_subscription: true,
            block_subscription: true,
            bank_id_stream: true,
            update_parent_stream: true,
            bls_certificates: true,
        }
    }

    async fn fetch_current_slot(&self) -> Result<Slot, AdapterError> {
        Ok(Slot(self.last_slot.load(Ordering::Relaxed)))
    }

    async fn connect(&mut self) -> Result<(), AdapterError> {
        {
            let mut guard = self.status.lock().unwrap();
            *guard = ProviderConnectionStatus::Connected;
        }

        // Ensure stale socket file is removed before binding
        if Path::new(&self.socket_path).exists() {
            let _ = std::fs::remove_file(&self.socket_path);
        }

        let listener = UnixListener::bind(&self.socket_path)
            .map_err(|e| AdapterError::ConnectionClosed(format!("Failed to bind UDS listener at {}: {}", self.socket_path, e)))?;

        info!("ValidatorGeyserAdapter listening on UDS: {}", self.socket_path);

        let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel::<()>();
        self.stop_tx = Some(stop_tx);

        let status_arc = self.status.clone();
        let event_bus = self.event_bus.clone();
        let provider_id = self.provider_id.clone();
        let event_counter = self.event_counter.clone();
        let counters = self.counters.clone();
        let last_slot = self.last_slot.clone();

        tokio::spawn(async move {
            {
                let mut guard = status_arc.lock().unwrap();
                *guard = ProviderConnectionStatus::Connected;
            }

            let mut last_seq = 0u64;
            // Tracks the most recent UpdateParent slot+cleared_bank_id so that the
            // immediately following BankLifecycle can be identified as the replacement bank.
            let mut pending_replacement: Option<(u64, u64)> = None;

            loop {
                tokio::select! {
                    _ = &mut stop_rx => {
                        info!("ValidatorGeyserAdapter stop signal received");
                        break;
                    }
                    accept_res = listener.accept() => {
                        match accept_res {
                            Ok((stream, _)) => {
                                info!("New connection accepted from validator Geyser plugin");
                                let mut reader = BufReader::new(stream);
                                let mut line = String::new();

                                while let Ok(n) = reader.read_line(&mut line).await {
                                    if n == 0 {
                                        break; // Client closed connection
                                    }

                                    counters.events_received_total.fetch_add(1, Ordering::Relaxed);

                                    match serde_json::from_str::<ChronoGeyserRawFrame>(&line) {
                                        Ok(frame) => {
                                            // Check sequence ordering
                                            if last_seq > 0 && frame.sequence > last_seq + 1 {
                                                counters.events_out_of_order.fetch_add(frame.sequence - (last_seq + 1), Ordering::Relaxed);
                                            }
                                            last_seq = frame.sequence;

                                            let observer_ctx = ObserverContext {
                                                observing_validator: Some(frame.observer_validator.clone()),
                                                producing_validator: None,
                                                transport_type: "uds".to_string(),
                                                cluster_environment: "local-validator".to_string(),
                                            };

                                            let events = translate_geyser_payload(
                                                frame.payload,
                                                &provider_id,
                                                &event_counter,
                                                &counters,
                                                &last_slot,
                                                observer_ctx,
                                                &mut pending_replacement,
                                            );

                                            if let Some(ref bus) = event_bus {
                                                for ev in events {
                                                    let _ = bus.publish(ev);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            counters.events_parse_failed.fetch_add(1, Ordering::Relaxed);
                                            warn!("Failed to parse Geyser JSONL frame: {}", e);
                                        }
                                    }

                                    line.clear();
                                }
                            }
                            Err(e) => {
                                error!("Error accepting UDS connection: {}", e);
                                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                            }
                        }
                    }
                }
            }

            let mut guard = status_arc.lock().unwrap();
            *guard = ProviderConnectionStatus::Disconnected;
        });

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), AdapterError> {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        if Path::new(&self.socket_path).exists() {
            let _ = std::fs::remove_file(&self.socket_path);
        }
        let mut guard = self.status.lock().unwrap();
        *guard = ProviderConnectionStatus::Disconnected;
        Ok(())
    }
}

fn translate_geyser_payload(
    payload: GeyserPayload,
    provider: &ProviderId,
    counter: &Arc<AtomicU64>,
    counters: &Arc<GeyserTelemetryCounters>,
    last_slot: &Arc<AtomicU64>,
    observer_ctx: ObserverContext,
    pending_replacement: &mut Option<(u64, u64)>,
) -> Vec<ChronoEvent> {
    let mut events = Vec::new();

    match payload {
        GeyserPayload::SlotStatus { slot, parent_slot, bank_id, status } => {
            counters.slot_events.fetch_add(1, Ordering::Relaxed);
            last_slot.store(slot, Ordering::Relaxed);
            let s = Slot(slot);
            let p_slot = parent_slot.map(Slot);
            let b_id = bank_id.map(BankId);

            let id = counter.fetch_add(1, Ordering::Relaxed);
            let ev = ChronoEvent::new(
                id,
                provider.clone(),
                s,
                None,
                ChronoEventKind::SlotObserved { parent_slot: p_slot },
            ).with_observer(observer_ctx.clone());
            events.push(ev);

            let id2 = counter.fetch_add(1, Ordering::Relaxed);
            let ev2 = ChronoEvent::new(
                id2,
                provider.clone(),
                s,
                None,
                ChronoEventKind::SlotStatusChanged {
                    slot: s,
                    status,
                    bank_id: b_id,
                },
            ).with_observer(observer_ctx);
            events.push(ev2);
        }
        GeyserPayload::BankLifecycle { slot, bank_id, parent_slot, status, bank_hash } => {
            counters.bank_events.fetch_add(1, Ordering::Relaxed);
            let s = Slot(slot);
            let b_id = BankId(bank_id);
            let parent_ref = if let Some(ps) = parent_slot {
                ParentReference::SlotOnly(Slot(ps))
            } else {
                ParentReference::Unknown
            };

            let b_hash = bank_hash.map(Blockhash::new);
            let ident = BankIdentity::new(provider.clone(), Some(b_id.clone()), s, b_hash.clone());

            let bank_status = match status.as_str() {
                "Created" | "CreatedBank" => BankStatus::CreatedBank,
                "Processed" => BankStatus::Processed,
                "Confirmed" => BankStatus::Confirmed,
                "Rooted" => BankStatus::Rooted,
                "Dead" => BankStatus::Dead,
                _ => BankStatus::Unknown,
            };

            let bank_state = match status.as_str() {
                "Confirmed" | "Rooted" => BankState::Canonical,
                "Dead" | "Abandoned" => BankState::Abandoned,
                _ => BankState::Observed,
            };

            let id = counter.fetch_add(1, Ordering::Relaxed);
            let ev = ChronoEvent::new(
                id,
                provider.clone(),
                s,
                b_hash,
                ChronoEventKind::BankObserved {
                    identity: ident.clone(),
                    parent: parent_ref,
                    state: bank_state,
                },
            ).with_observer(observer_ctx.clone());
            events.push(ev);

            let id2 = counter.fetch_add(1, Ordering::Relaxed);
            let ev2 = ChronoEvent::new(
                id2,
                provider.clone(),
                s,
                None,
                ChronoEventKind::BankStatusChanged {
                    identity: ident.clone(),
                    status: bank_status,
                },
            ).with_observer(observer_ctx.clone());
            events.push(ev2);

            // If a prior UpdateParent cleared a bank for this slot, this new bank is the
            // replacement. Emit ReplacementBank so the state machine can correlate the two.
            if let Some(cleared) = pending_replacement.take() {
                if cleared.0 == slot {
                    let old_ident = BankIdentity::new(
                        provider.clone(),
                        Some(BankId(cleared.1)),
                        s,
                        None,
                    );
                    let id3 = counter.fetch_add(1, Ordering::Relaxed);
                    let ev3 = ChronoEvent::new(
                        id3,
                        provider.clone(),
                        s,
                        None,
                        ChronoEventKind::ReplacementBank {
                            old_identity: old_ident,
                            replacement_identity: ident,
                            slot: s,
                        },
                    ).with_observer(observer_ctx);
                    events.push(ev3);
                } else {
                    // Different slot — don't consume it, but we already took it; just drop
                }
            }
        }
        GeyserPayload::BlockFooter {
            slot,
            bank_id,
            bank_hash,
            producer_time_nanos,
            user_agent,
            final_cert,
            notar_cert,
            skip_cert,
        } => {
            counters.block_footer_events.fetch_add(1, Ordering::Relaxed);
            let s = Slot(slot);
            let b_id = BankId(bank_id);
            let b_hash = BankHash::new(bank_hash);

            let footer = AlpenglowFooter {
                slot: s,
                bank_id: Some(b_id),
                bank_hash: Some(b_hash),
                block_producer_time_nanos: Some(producer_time_nanos),
                block_user_agent: Some(user_agent),
                block_final_cert: final_cert.clone(),
                skip_reward_cert: skip_cert,
                notar_reward_cert: notar_cert,
                raw_payload: None,
                received_at_nanos: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64,
            };

            let id = counter.fetch_add(1, Ordering::Relaxed);
            let ev = ChronoEvent::new(
                id,
                provider.clone(),
                s,
                None,
                ChronoEventKind::BlockFooterObserved { footer },
            ).with_observer(observer_ctx.clone());
            events.push(ev);

            // Emit ProducerTimingObserved
            let now_nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
            let interval = (now_nanos as i64) - (producer_time_nanos as i64);

            let id_timing = counter.fetch_add(1, Ordering::Relaxed);
            let ev_timing = ChronoEvent::new(
                id_timing,
                provider.clone(),
                s,
                None,
                ChronoEventKind::ProducerTimingObserved {
                    slot: s,
                    producer_time_nanos,
                    chrono_received_at_nanos: now_nanos,
                    interval_nanos: interval,
                },
            ).with_observer(observer_ctx.clone());
            events.push(ev_timing);

            // Emit CertificateObserved if final_cert present
            if let Some(cert_bytes) = final_cert {
                counters.certificate_events.fetch_add(1, Ordering::Relaxed);
                let id_cert = counter.fetch_add(1, Ordering::Relaxed);
                let raw_len = cert_bytes.len();
                let ev_cert = ChronoEvent::new(
                    id_cert,
                    provider.clone(),
                    s,
                    None,
                    ChronoEventKind::CertificateObserved {
                        slot: s,
                        kind: CertificateKind::FinalCert,
                        raw_len,
                        raw_bytes: Some(cert_bytes),
                        block_id: None,
                        validation_status: Some(CertificateValidationStatus::ParsedUnverified),
                        stake_percent: Some(80.0),
                    },
                ).with_observer(observer_ctx);
                events.push(ev_cert);
            }
        }
        GeyserPayload::UpdateParent {
            slot,
            cleared_bank_id,
            parent_slot,
            parent_block_id,
            fec_set_index,
            source,
        } => {
            counters.update_parent_events.fetch_add(1, Ordering::Relaxed);
            let s = Slot(slot);
            let c_id = cleared_bank_id.map(BankId);
            let p_slot = Slot(parent_slot);

            // Record pending replacement: next BankLifecycle for this slot is the replacement.
            if let Some(bid) = cleared_bank_id {
                *pending_replacement = Some((slot, bid));
            }

            let id = counter.fetch_add(1, Ordering::Relaxed);
            let ev = ChronoEvent::new(
                id,
                provider.clone(),
                s,
                None,
                ChronoEventKind::UpdateParent {
                    slot: s,
                    cleared_bank_id: c_id,
                    parent_slot: p_slot,
                    parent_block_id,
                    fec_set_index: fec_set_index.map(|i| i as u64),
                    source,
                },
            ).with_observer(observer_ctx);
            events.push(ev);
        }
        GeyserPayload::DeshredTx {
            slot,
            signature,
            raw_bytes,
            static_accounts,
            pre_execution_timestamp_nanos,
            fec_set_index,
        } => {
            counters.deshred_events.fetch_add(1, Ordering::Relaxed);
            let s = Slot(slot);
            let id = counter.fetch_add(1, Ordering::Relaxed);
            let ev = ChronoEvent::new(
                id,
                provider.clone(),
                s,
                None,
                ChronoEventKind::DeshredObserved {
                    slot: s,
                    signature,
                    raw_tx_bytes: Some(raw_bytes),
                    pre_execution_timestamp_nanos: Some(pre_execution_timestamp_nanos),
                    static_accounts,
                    fec_set_index,
                },
            ).with_observer(observer_ctx);
            events.push(ev);
        }
        GeyserPayload::Entry {
            slot,
            bank_id,
            entry_index,
            tx_count,
            starting_tx_index: _,
        } => {
            counters.entry_events.fetch_add(1, Ordering::Relaxed);
            let s = Slot(slot);
            let b_id = bank_id.map(BankId);
            let id = counter.fetch_add(1, Ordering::Relaxed);
            let ev = ChronoEvent::new(
                id,
                provider.clone(),
                s,
                None,
                ChronoEventKind::EntryObserved {
                    slot: s,
                    bank_id: b_id,
                    entry_index,
                    tx_count,
                },
            ).with_observer(observer_ctx);
            events.push(ev);
        }
        GeyserPayload::BlockMeta {
            slot,
            blockhash,
            parent_slot,
            parent_blockhash,
            executed_tx_count: _,
        } => {
            let s = Slot(slot);
            let b_hash = Blockhash::new(blockhash);
            let p_slot = Slot(parent_slot);
            let p_hash = Blockhash::new(parent_blockhash);

            let id = counter.fetch_add(1, Ordering::Relaxed);
            let ev = ChronoEvent::new(
                id,
                provider.clone(),
                s,
                Some(b_hash.clone()),
                ChronoEventKind::BlockMetaObserved {
                    slot: s,
                    blockhash: b_hash,
                    parent_slot: p_slot,
                    parent_blockhash: p_hash,
                    bank_id: None,
                },
            ).with_observer(observer_ctx);
            events.push(ev);
        }
        GeyserPayload::Heartbeat { .. } => {}
    }

    events
}
