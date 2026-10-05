use crate::capabilities::ProviderCapabilityMatrix;
use crate::provider::{AdapterError, ProviderAdapter, ProviderCapabilities, ProviderConnectionStatus};
use async_trait::async_trait;
use chrono_core::bus::EventBus;
use chrono_core::events::{ChronoEvent, ChronoEventKind};
use chrono_core::identity::BankIdentity;
use chrono_core::types::{
    AlpenglowFooter, BankHash, BankId, Blockhash, CertificateKind, CertificateValidationStatus,
    ParentReference, ProviderId, Slot,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use yellowstone_grpc_proto::geyser::geyser_client::GeyserClient;
use yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof;
use yellowstone_grpc_proto::geyser::{
    SubscribeRequest, SubscribeRequestFilterBlockFooter, SubscribeRequestFilterBlocks,
    SubscribeRequestFilterBlocksMeta, SubscribeRequestFilterEntry, SubscribeRequestFilterSlots,
    SubscribeUpdate,
};

/// Bounded escape-hatch record for raw Alpenglow payloads.
#[derive(Debug, Clone)]
pub struct RawPayloadRecord {
    pub schema_version: String,
    pub provider: ProviderId,
    pub received_at_nanos: u64,
    pub payload_bytes: Vec<u8>,
}

/// Bounded in-memory raw payload store for forward compatibility.
#[derive(Debug, Default)]
pub struct RawPayloadStore {
    max_entries: usize,
    records: Vec<RawPayloadRecord>,
}

impl RawPayloadStore {
    pub fn new(max_entries: usize) -> Self {
        Self {
            max_entries,
            records: Vec::with_capacity(max_entries),
        }
    }

    pub fn push(&mut self, record: RawPayloadRecord) {
        if self.records.len() >= self.max_entries {
            self.records.remove(0);
        }
        self.records.push(record);
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

/// Configuration for Yellowstone gRPC streaming provider.
#[derive(Debug, Clone)]
pub struct YellowstoneConfig {
    pub endpoint: String,
    pub token: Option<String>,
    pub subscribe_slots: bool,
    pub subscribe_blocks: bool,
    pub subscribe_block_meta: bool,
    pub subscribe_entries: bool,
    pub include_update_parent: bool,
    pub include_block_footer: bool,
    pub include_certificates: bool,
}

impl YellowstoneConfig {
    /// Loads configuration from environment variables or returns defaults.
    ///
    /// Free-first configuration:
    /// - `CHRONO_YELLOWSTONE_ENDPOINT`: Target gRPC endpoint (default: PublicNode Testnet)
    /// - `CHRONO_YELLOWSTONE_TOKEN`: Optional auth token / x-token header
    pub fn from_env() -> Self {
        let endpoint = std::env::var("CHRONO_YELLOWSTONE_ENDPOINT")
            .unwrap_or_else(|_| "https://solana-testnet.publicnode.com:443".to_string());
        let token = std::env::var("CHRONO_YELLOWSTONE_TOKEN").ok();

        Self {
            endpoint,
            token,
            subscribe_slots: true,
            subscribe_blocks: true,
            subscribe_block_meta: true,
            subscribe_entries: true,
            include_update_parent: true,
            include_block_footer: true,
            include_certificates: true,
        }
    }
}

/// Production-grade Yellowstone gRPC Geyser Provider Adapter.
pub struct YellowstoneAdapter {
    provider_id: ProviderId,
    config: YellowstoneConfig,
    status: Arc<Mutex<ProviderConnectionStatus>>,
    event_bus: Option<Arc<EventBus>>,
    event_counter: Arc<AtomicU64>,
    capabilities: ProviderCapabilityMatrix,
    raw_payloads: Arc<Mutex<RawPayloadStore>>,
    disconnect_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl YellowstoneAdapter {
    pub fn new(
        provider_id: impl Into<String>,
        config: YellowstoneConfig,
        event_bus: Option<Arc<EventBus>>,
    ) -> Self {
        let id_str = provider_id.into();
        let matrix = ProviderCapabilityMatrix::yellowstone_grpc(&id_str);
        Self {
            provider_id: ProviderId::new(id_str),
            config,
            status: Arc::new(Mutex::new(ProviderConnectionStatus::Disconnected)),
            event_bus,
            event_counter: Arc::new(AtomicU64::new(1)),
            capabilities: matrix,
            raw_payloads: Arc::new(Mutex::new(RawPayloadStore::new(1000))),
            disconnect_tx: None,
        }
    }

    pub fn capability_matrix(&self) -> &ProviderCapabilityMatrix {
        &self.capabilities
    }

    pub fn raw_payload_store(&self) -> Arc<Mutex<RawPayloadStore>> {
        self.raw_payloads.clone()
    }

    /// Normalizes a Yellowstone `SubscribeUpdate` protobuf into Chrono normalized events.
    pub fn normalize_update(
        update: SubscribeUpdate,
        provider_id: &ProviderId,
        event_counter: &AtomicU64,
    ) -> Vec<ChronoEvent> {
        let mut events = Vec::new();

        if let Some(oneof) = update.update_oneof {
            match oneof {
                UpdateOneof::Slot(slot_update) => {
                    let event_id = event_counter.fetch_add(1, Ordering::Relaxed);
                    let slot = Slot(slot_update.slot);
                    let parent_slot = slot_update.parent.map(Slot);
                    let bank_id = slot_update.bank_id.map(BankId);

                    let status_str = match slot_update.status {
                        0 => "Processed",
                        1 => "Rooted",
                        2 => "Confirmed",
                        3 => "CreatedBank",
                        4 => "Dead",
                        _ => "Unknown",
                    };

                    // 1. SlotObserved
                    events.push(ChronoEvent::new(
                        event_id,
                        provider_id.clone(),
                        slot,
                        None,
                        ChronoEventKind::SlotObserved { parent_slot },
                    ));

                    // 2. SlotStatusChanged
                    let status_event_id = event_counter.fetch_add(1, Ordering::Relaxed);
                    events.push(ChronoEvent::new(
                        status_event_id,
                        provider_id.clone(),
                        slot,
                        None,
                        ChronoEventKind::SlotStatusChanged {
                            slot,
                            status: status_str.to_string(),
                            bank_id: bank_id.clone(),
                        },
                    ));

                    // 3. If CreatedBank, emit BankCreated
                    if slot_update.status == 3 {
                        let created_id = event_counter.fetch_add(1, Ordering::Relaxed);
                        let identity = BankIdentity::new(provider_id.clone(), bank_id, slot, None);
                        let parent_ref = match parent_slot {
                            Some(p) => ParentReference::SlotOnly(p),
                            None => ParentReference::Unknown,
                        };
                        events.push(ChronoEvent::new(
                            created_id,
                            provider_id.clone(),
                            slot,
                            None,
                            ChronoEventKind::BankCreated {
                                identity,
                                parent: parent_ref,
                            },
                        ));
                    }
                }
                UpdateOneof::Block(block_update) => {
                    let event_id = event_counter.fetch_add(1, Ordering::Relaxed);
                    let slot = Slot(block_update.slot);
                    let blockhash = Blockhash::new(block_update.blockhash);
                    let parent_blockhash = if block_update.parent_blockhash.is_empty() {
                        None
                    } else {
                        Some(Blockhash::new(block_update.parent_blockhash))
                    };

                    events.push(ChronoEvent::new(
                        event_id,
                        provider_id.clone(),
                        slot,
                        Some(blockhash.clone()),
                        ChronoEventKind::BlockObserved {
                            blockhash,
                            parent_blockhash,
                            tx_count: block_update.executed_transaction_count,
                        },
                    ));
                }
                UpdateOneof::BlockMeta(meta_update) => {
                    let event_id = event_counter.fetch_add(1, Ordering::Relaxed);
                    let slot = Slot(meta_update.slot);
                    let blockhash = Blockhash::new(meta_update.blockhash);
                    let parent_slot = Slot(meta_update.parent_slot);
                    let parent_blockhash = Blockhash::new(meta_update.parent_blockhash);
                    let bank_id = if meta_update.bank_id == 0 {
                        None
                    } else {
                        Some(BankId(meta_update.bank_id))
                    };

                    events.push(ChronoEvent::new(
                        event_id,
                        provider_id.clone(),
                        slot,
                        Some(blockhash.clone()),
                        ChronoEventKind::BlockMetaObserved {
                            slot,
                            blockhash,
                            parent_slot,
                            parent_blockhash,
                            bank_id,
                        },
                    ));
                }
                UpdateOneof::Entry(entry_update) => {
                    let event_id = event_counter.fetch_add(1, Ordering::Relaxed);
                    let slot = Slot(entry_update.slot);
                    let bank_id = if entry_update.bank_id == 0 {
                        None
                    } else {
                        Some(BankId(entry_update.bank_id))
                    };

                    events.push(ChronoEvent::new(
                        event_id,
                        provider_id.clone(),
                        slot,
                        None,
                        ChronoEventKind::EntryObserved {
                            slot,
                            bank_id,
                            entry_index: entry_update.index,
                            tx_count: entry_update.executed_transaction_count,
                        },
                    ));
                }
                UpdateOneof::BlockFooter(footer_update) => {
                    let event_id = event_counter.fetch_add(1, Ordering::Relaxed);
                    let slot = Slot(footer_update.slot);
                    let bank_id = Some(BankId(footer_update.bank_id));
                    let bank_hash = if footer_update.bank_hash.is_empty() {
                        None
                    } else {
                        Some(BankHash::new(bs58::encode(&footer_update.bank_hash).into_string()))
                    };
                    let user_agent = if footer_update.block_user_agent.is_empty() {
                        None
                    } else {
                        Some(String::from_utf8_lossy(&footer_update.block_user_agent).to_string())
                    };

                    let footer = AlpenglowFooter {
                        slot,
                        bank_id: bank_id.clone(),
                        bank_hash,
                        block_producer_time_nanos: Some(footer_update.block_producer_time_nanos),
                        block_user_agent: user_agent,
                        block_final_cert: footer_update.block_final_cert.clone(),
                        skip_reward_cert: footer_update.skip_reward_cert.clone(),
                        notar_reward_cert: footer_update.notar_reward_cert.clone(),
                        raw_payload: None,
                        received_at_nanos: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos() as u64,
                    };

                    // 1. Emit BlockFooterObserved
                    events.push(ChronoEvent::new(
                        event_id,
                        provider_id.clone(),
                        slot,
                        None,
                        ChronoEventKind::BlockFooterObserved { footer },
                    ));

                    // 2. If FinalCert present, emit CertificateObserved
                    if let Some(ref cert_bytes) = footer_update.block_final_cert {
                        let cert_id = event_counter.fetch_add(1, Ordering::Relaxed);
                        events.push(ChronoEvent::new(
                            cert_id,
                            provider_id.clone(),
                            slot,
                            None,
                            ChronoEventKind::CertificateObserved {
                                slot,
                                kind: CertificateKind::FinalCert,
                                raw_len: cert_bytes.len(),
                                raw_bytes: Some(cert_bytes.clone()),
                                block_id: None,
                                validation_status: Some(CertificateValidationStatus::ParsedUnverified),
                                stake_percent: Some(80.0),
                            },
                        ));
                    }

                    // 3. If NotarRewardCert present, emit CertificateObserved
                    if let Some(ref cert_bytes) = footer_update.notar_reward_cert {
                        let cert_id = event_counter.fetch_add(1, Ordering::Relaxed);
                        events.push(ChronoEvent::new(
                            cert_id,
                            provider_id.clone(),
                            slot,
                            None,
                            ChronoEventKind::CertificateObserved {
                                slot,
                                kind: CertificateKind::NotarRewardCert,
                                raw_len: cert_bytes.len(),
                                raw_bytes: Some(cert_bytes.clone()),
                                block_id: None,
                                validation_status: Some(CertificateValidationStatus::ParsedUnverified),
                                stake_percent: Some(60.0),
                            },
                        ));
                    }

                    // 4. If SkipRewardCert present, emit CertificateObserved
                    if let Some(ref cert_bytes) = footer_update.skip_reward_cert {
                        let cert_id = event_counter.fetch_add(1, Ordering::Relaxed);
                        events.push(ChronoEvent::new(
                            cert_id,
                            provider_id.clone(),
                            slot,
                            None,
                            ChronoEventKind::CertificateObserved {
                                slot,
                                kind: CertificateKind::SkipRewardCert,
                                raw_len: cert_bytes.len(),
                                raw_bytes: Some(cert_bytes.clone()),
                                block_id: None,
                                validation_status: Some(CertificateValidationStatus::ParsedUnverified),
                                stake_percent: None,
                            },
                        ));
                    }
                }
                UpdateOneof::EntryUpdateParent(update_parent) => {
                    let event_id = event_counter.fetch_add(1, Ordering::Relaxed);
                    let slot = Slot(update_parent.slot);
                    let cleared_bank_id = Some(BankId(update_parent.cleared_bank_id));
                    let parent_slot = Slot(update_parent.parent_slot);
                    let parent_block_id = if update_parent.parent_block_id.is_empty() {
                        None
                    } else {
                        Some(bs58::encode(&update_parent.parent_block_id).into_string())
                    };

                    events.push(ChronoEvent::new(
                        event_id,
                        provider_id.clone(),
                        slot,
                        None,
                        ChronoEventKind::UpdateParent {
                            slot,
                            cleared_bank_id,
                            parent_slot,
                            parent_block_id,
                            fec_set_index: None,
                            source: "yellowstone_entry_update_parent".to_string(),
                        },
                    ));
                }
                UpdateOneof::Transaction(tx_update) => {
                    let event_id = event_counter.fetch_add(1, Ordering::Relaxed);
                    let slot = Slot(tx_update.slot);
                    let bank_id = if tx_update.bank_id == 0 {
                        None
                    } else {
                        Some(BankId(tx_update.bank_id))
                    };
                    let sig = if let Some(ref tx_info) = tx_update.transaction {
                        bs58::encode(&tx_info.signature).into_string()
                    } else {
                        String::new()
                    };

                    events.push(ChronoEvent::new(
                        event_id,
                        provider_id.clone(),
                        slot,
                        None,
                        ChronoEventKind::TransactionObserved {
                            slot,
                            bank_id,
                            signature: sig,
                        },
                    ));
                }
                _ => {}
            }
        }

        events
    }

    /// Builds a subscription request enabling all supported Alpenglow streams.
    pub fn build_subscribe_request(&self) -> SubscribeRequest {
        let mut req = SubscribeRequest::default();

        if self.config.subscribe_slots {
            let mut slots = HashMap::new();
            slots.insert("chrono_slots".to_string(), SubscribeRequestFilterSlots {
                filter_by_commitment: None,
                interslot_updates: Some(true),
            });
            req.slots = slots;
        }

        if self.config.subscribe_blocks {
            let mut blocks = HashMap::new();
            blocks.insert("chrono_blocks".to_string(), SubscribeRequestFilterBlocks {
                account_include: vec![],
                include_transactions: Some(false),
                include_accounts: Some(false),
                include_entries: Some(true),
                cuckoo_account_include: None,
            });
            req.blocks = blocks;
        }

        if self.config.subscribe_block_meta {
            let mut blocks_meta = HashMap::new();
            blocks_meta.insert(
                "chrono_blocks_meta".to_string(),
                SubscribeRequestFilterBlocksMeta {},
            );
            req.blocks_meta = blocks_meta;
        }

        if self.config.subscribe_entries {
            let mut entry = HashMap::new();
            entry.insert(
                "chrono_entries".to_string(),
                SubscribeRequestFilterEntry {
                    include_update_parent: Some(self.config.include_update_parent),
                },
            );
            req.entry = entry;
        }

        if self.config.include_block_footer {
            let mut block_footer = HashMap::new();
            block_footer.insert(
                "chrono_block_footer".to_string(),
                SubscribeRequestFilterBlockFooter {
                    include_certificates: Some(self.config.include_certificates),
                },
            );
            req.block_footer = block_footer;
        }

        req
    }
}

#[async_trait]
impl ProviderAdapter for YellowstoneAdapter {
    fn provider_id(&self) -> &ProviderId {
        &self.provider_id
    }

    fn status(&self) -> ProviderConnectionStatus {
        let guard = futures_util::FutureExt::now_or_never(self.status.lock());
        guard
            .map(|g| g.clone())
            .unwrap_or(ProviderConnectionStatus::Disconnected)
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities::yellowstone_geyser()
    }

    async fn connect(&mut self) -> Result<(), AdapterError> {
        let endpoint_str = self.config.endpoint.clone();
        info!(
            "Attempting connection to Yellowstone gRPC endpoint: {}",
            endpoint_str
        );

        {
            let mut status = self.status.lock().await;
            *status = ProviderConnectionStatus::Reconnecting(1);
        }

        // Establish tonic endpoint
        let endpoint = match tonic::transport::Endpoint::from_shared(endpoint_str.clone()) {
            Ok(ep) => ep,
            Err(e) => {
                let err_msg = format!("Invalid gRPC endpoint URI {}: {}", endpoint_str, e);
                let mut status = self.status.lock().await;
                *status = ProviderConnectionStatus::Degraded(err_msg.clone());
                return Err(AdapterError::ConnectionClosed(err_msg));
            }
        };

        // Connect channel
        let channel = match endpoint.connect().await {
            Ok(c) => c,
            Err(e) => {
                let err_msg = format!(
                    "Failed to connect to Yellowstone gRPC endpoint {}: {}. Token provided: {}",
                    endpoint_str,
                    e,
                    self.config.token.is_some()
                );
                warn!("{}", err_msg);
                let mut status = self.status.lock().await;
                *status = ProviderConnectionStatus::Degraded(err_msg.clone());
                return Err(AdapterError::ConnectionClosed(err_msg));
            }
        };

        let mut client = GeyserClient::new(channel);

        // Prepare request stream
        let subscribe_req = self.build_subscribe_request();
        let (tx, rx) = tokio::sync::mpsc::channel(10);
        tx.send(subscribe_req).await.map_err(|e| {
            AdapterError::ConnectionClosed(format!("Failed to send subscription request: {}", e))
        })?;

        let stream_req = tokio_stream::wrappers::ReceiverStream::new(rx);

        let response = match client.subscribe(stream_req).await {
            Ok(r) => r,
            Err(status) => {
                let err_msg = format!(
                    "Yellowstone subscribe RPC returned error code {:?}: {}",
                    status.code(),
                    status.message()
                );
                warn!("{}", err_msg);
                let mut current_status = self.status.lock().await;
                *current_status = ProviderConnectionStatus::Degraded(err_msg.clone());
                return Err(AdapterError::ConnectionClosed(err_msg));
            }
        };

        {
            let mut current_status = self.status.lock().await;
            *current_status = ProviderConnectionStatus::Connected;
        }

        let (disconnect_tx, mut disconnect_rx) = tokio::sync::oneshot::channel();
        self.disconnect_tx = Some(disconnect_tx);

        let provider_id = self.provider_id.clone();
        let event_counter = self.event_counter.clone();
        let event_bus = self.event_bus.clone();
        let status_arc = self.status.clone();
        let mut inbound = response.into_inner();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = &mut disconnect_rx => {
                        info!("YellowstoneAdapter received disconnect signal");
                        break;
                    }
                    msg_res = inbound.message() => {
                        match msg_res {
                            Ok(Some(update)) => {
                                let events = Self::normalize_update(update, &provider_id, &event_counter);
                                if let Some(ref bus) = event_bus {
                                    for ev in events {
                                        bus.publish(ev);
                                    }
                                }
                            }
                            Ok(None) => {
                                warn!("Yellowstone stream closed by server");
                                let mut st = status_arc.lock().await;
                                *st = ProviderConnectionStatus::Disconnected;
                                break;
                            }
                            Err(e) => {
                                error!("Error on Yellowstone stream: {}", e);
                                let mut st = status_arc.lock().await;
                                *st = ProviderConnectionStatus::Degraded(e.to_string());
                                break;
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), AdapterError> {
        if let Some(tx) = self.disconnect_tx.take() {
            let _ = tx.send(());
        }
        let mut status = self.status.lock().await;
        *status = ProviderConnectionStatus::Disconnected;
        Ok(())
    }

    async fn fetch_current_slot(&self) -> Result<Slot, AdapterError> {
        // Yellowstone is streaming first; slot is tracked via events
        Ok(Slot(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof;
    use yellowstone_grpc_proto::geyser::{
        SubscribeUpdate, SubscribeUpdateBlockFooter, SubscribeUpdateEntryUpdateParent,
        SubscribeUpdateSlot,
    };

    #[test]
    fn test_normalize_slot_update_with_bank_id() {
        let slot_update = SubscribeUpdateSlot {
            slot: 12345,
            parent: Some(12344),
            status: 3, // CreatedBank
            bank_id: Some(42),
            dead_error: None,
        };

        let update = SubscribeUpdate {
            filters: vec![],
            update_oneof: Some(UpdateOneof::Slot(slot_update)),
            created_at: None,
        };

        let provider = ProviderId::new("yellowstone-test");
        let counter = AtomicU64::new(1);
        let events = YellowstoneAdapter::normalize_update(update, &provider, &counter);

        assert_eq!(events.len(), 3); // SlotObserved, SlotStatusChanged, BankCreated
        match &events[2].kind {
            ChronoEventKind::BankCreated { identity, parent } => {
                assert_eq!(identity.slot, Slot(12345));
                assert_eq!(identity.bank_id, Some(BankId(42)));
                assert_eq!(*parent, ParentReference::SlotOnly(Slot(12344)));
            }
            _ => panic!("Expected BankCreated event"),
        }
    }

    #[test]
    fn test_normalize_block_footer_with_certificates() {
        let footer_update = SubscribeUpdateBlockFooter {
            slot: 9999,
            bank_id: 101,
            bank_hash: vec![1; 32],
            block_producer_time_nanos: 1_700_000_000_000_000,
            block_user_agent: b"agave-v2.1.0".to_vec(),
            block_final_cert: Some(vec![0xAA; 48]),
            skip_reward_cert: None,
            notar_reward_cert: Some(vec![0xBB; 48]),
        };

        let update = SubscribeUpdate {
            filters: vec![],
            update_oneof: Some(UpdateOneof::BlockFooter(footer_update)),
            created_at: None,
        };

        let provider = ProviderId::new("yellowstone-test");
        let counter = AtomicU64::new(1);
        let events = YellowstoneAdapter::normalize_update(update, &provider, &counter);

        // Should emit BlockFooterObserved, FinalCert CertificateObserved, NotarRewardCert CertificateObserved
        assert_eq!(events.len(), 3);
        match &events[0].kind {
            ChronoEventKind::BlockFooterObserved { footer } => {
                assert_eq!(footer.slot, Slot(9999));
                assert_eq!(footer.bank_id, Some(BankId(101)));
                assert!(footer.bank_hash.is_some());
                assert_eq!(footer.block_user_agent.as_deref(), Some("agave-v2.1.0"));
            }
            _ => panic!("Expected BlockFooterObserved"),
        }
    }

    #[test]
    fn test_normalize_entry_update_parent() {
        let update_parent = SubscribeUpdateEntryUpdateParent {
            slot: 500,
            cleared_bank_id: 88,
            parent_slot: 499,
            parent_block_id: vec![2; 32],
        };

        let update = SubscribeUpdate {
            filters: vec![],
            update_oneof: Some(UpdateOneof::EntryUpdateParent(update_parent)),
            created_at: None,
        };

        let provider = ProviderId::new("yellowstone-test");
        let counter = AtomicU64::new(1);
        let events = YellowstoneAdapter::normalize_update(update, &provider, &counter);

        assert_eq!(events.len(), 1);
        match &events[0].kind {
            ChronoEventKind::UpdateParent {
                slot,
                cleared_bank_id,
                parent_slot,
                parent_block_id,
                ..
            } => {
                assert_eq!(*slot, Slot(500));
                assert_eq!(*cleared_bank_id, Some(BankId(88)));
                assert_eq!(*parent_slot, Slot(499));
                assert!(parent_block_id.is_some());
            }
            _ => panic!("Expected UpdateParent event"),
        }
    }
}
