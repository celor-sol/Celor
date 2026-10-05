use crate::config::PluginConfig;
use crate::events::{ChronoGeyserRawFrame, GeyserPayload};
use crate::ipc::GeyserIpcWriter;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};

/// High-fidelity Chrono Geyser Plugin implementation.
#[derive(Debug)]
pub struct ChronoGeyserPlugin {
    config: PluginConfig,
    ipc_writer: Option<GeyserIpcWriter>,
    sequence_counter: Arc<AtomicU64>,
}

impl ChronoGeyserPlugin {
    pub fn new() -> Self {
        Self {
            config: PluginConfig::default(),
            ipc_writer: None,
            sequence_counter: Arc::new(AtomicU64::new(1)),
        }
    }

    pub fn with_config(config: PluginConfig) -> Self {
        let ipc_writer = Some(GeyserIpcWriter::new(
            config.uds_socket_path.clone(),
            config.channel_buffer_capacity,
        ));
        Self {
            config,
            ipc_writer,
            sequence_counter: Arc::new(AtomicU64::new(1)),
        }
    }

    pub fn load_config(&mut self, config_file: &str) -> Result<(), String> {
        info!("Loading Chrono Geyser Plugin from config: {}", config_file);
        let config = if let Ok(data) = std::fs::read_to_string(config_file) {
            serde_json::from_str::<PluginConfig>(&data).unwrap_or_else(|e| {
                warn!("Failed to parse config file ({}): {}. Using defaults.", config_file, e);
                PluginConfig::default()
            })
        } else {
            warn!("Config file {} not readable. Using defaults.", config_file);
            PluginConfig::default()
        };

        self.ipc_writer = Some(GeyserIpcWriter::new(
            config.uds_socket_path.clone(),
            config.channel_buffer_capacity,
        ));
        self.config = config;
        info!("Chrono Geyser Plugin initialized successfully.");
        Ok(())
    }

    pub fn on_load(&mut self, config_file: &str) -> Result<(), String> {
        self.load_config(config_file)
    }

    pub fn on_unload(&mut self) {
        info!("Chrono Geyser Plugin unloaded.");
        self.ipc_writer = None;
    }

    fn current_monotonic_nanos() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64
    }

    fn dispatch_payload(&self, payload: GeyserPayload) -> bool {
        if let Some(ref writer) = self.ipc_writer {
            let seq = self.sequence_counter.fetch_add(1, Ordering::Relaxed);
            let frame = ChronoGeyserRawFrame {
                sequence: seq,
                observer_validator: self.config.observer_validator_id.clone(),
                observed_at_nanos: Self::current_monotonic_nanos(),
                payload,
            };
            writer.dispatch(frame)
        } else {
            false
        }
    }

    /// Invoked upon bank lifecycle transition (Created, Processed, Confirmed, Rooted, Dead).
    pub fn update_bank_status(
        &self,
        slot: u64,
        parent_slot: Option<u64>,
        status: &str,
        bank_id: u64,
        bank_hash: Option<String>,
    ) {
        if self.config.enable_bank_status {
            self.dispatch_payload(GeyserPayload::BankLifecycle {
                slot,
                bank_id,
                parent_slot,
                status: status.to_string(),
                bank_hash,
            });
        }
    }

    /// Invoked upon receiving Alpenglow Block Footer.
    #[allow(clippy::too_many_arguments)]
    pub fn notify_block_footer(
        &self,
        slot: u64,
        bank_id: u64,
        bank_hash: String,
        producer_time_nanos: u64,
        user_agent: String,
        final_cert: Option<Vec<u8>>,
        notar_cert: Option<Vec<u8>>,
        skip_cert: Option<Vec<u8>>,
    ) {
        if self.config.enable_block_footer {
            self.dispatch_payload(GeyserPayload::BlockFooter {
                slot,
                bank_id,
                bank_hash,
                producer_time_nanos,
                user_agent,
                final_cert,
                notar_cert,
                skip_cert,
            });
        }
    }

    /// Invoked upon receiving fast leader handover UpdateParent marker.
    pub fn notify_update_parent(
        &self,
        slot: u64,
        cleared_bank_id: Option<u64>,
        parent_slot: u64,
        parent_block_id: Option<String>,
        fec_set_index: Option<u32>,
        source: &str,
    ) {
        if self.config.enable_update_parent {
            self.dispatch_payload(GeyserPayload::UpdateParent {
                slot,
                cleared_bank_id,
                parent_slot,
                parent_block_id,
                fec_set_index,
                source: source.to_string(),
            });
        }
    }

    /// Invoked upon pre-execution transaction reconstruction.
    pub fn notify_deshred(
        &self,
        slot: u64,
        signature: String,
        raw_bytes: Vec<u8>,
        static_accounts: Vec<String>,
        fec_set_index: Option<u64>,
    ) {
        if self.config.enable_deshred {
            self.dispatch_payload(GeyserPayload::DeshredTx {
                slot,
                signature,
                raw_bytes,
                static_accounts,
                pre_execution_timestamp_nanos: Self::current_monotonic_nanos(),
                fec_set_index,
            });
        }
    }

    /// Invoked upon entry formation.
    pub fn notify_entry(
        &self,
        slot: u64,
        bank_id: Option<u64>,
        entry_index: u64,
        tx_count: u64,
        starting_tx_index: u64,
    ) {
        if self.config.enable_entries {
            self.dispatch_payload(GeyserPayload::Entry {
                slot,
                bank_id,
                entry_index,
                tx_count,
                starting_tx_index,
            });
        }
    }

    /// Invoked upon block seal and metadata update.
    pub fn notify_block_metadata(
        &self,
        slot: u64,
        blockhash: String,
        parent_slot: u64,
        parent_blockhash: String,
        executed_tx_count: u64,
    ) {
        self.dispatch_payload(GeyserPayload::BlockMeta {
            slot,
            blockhash,
            parent_slot,
            parent_blockhash,
            executed_tx_count,
        });
    }

    pub fn stats(&self) -> (u64, u64, bool) {
        if let Some(ref writer) = self.ipc_writer {
            (writer.sent_count(), writer.dropped_count(), writer.is_connected())
        } else {
            (0, 0, false)
        }
    }
}

impl Default for ChronoGeyserPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl agave_geyser_plugin_interface::geyser_plugin_interface::GeyserPlugin for ChronoGeyserPlugin {
    fn name(&self) -> &'static str {
        "chrono-geyser-plugin"
    }

    fn on_load(
        &mut self,
        config_file: &str,
        _is_reload: bool,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        self.load_config(config_file).map_err(|msg| {
            agave_geyser_plugin_interface::geyser_plugin_interface::GeyserPluginError::ConfigFileReadError { msg }
        })?;
        Ok(())
    }

    fn on_unload(&mut self) {
        self.ipc_writer = None;
    }

    fn update_slot_status(
        &self,
        slot: solana_clock::Slot,
        parent: Option<u64>,
        status: &agave_geyser_plugin_interface::geyser_plugin_interface::SlotStatus,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        self.dispatch_payload(GeyserPayload::SlotStatus {
            slot,
            parent_slot: parent,
            bank_id: None,
            status: status.as_str().to_string(),
        });
        Ok(())
    }

    fn update_bank_status(
        &self,
        slot: solana_clock::Slot,
        parent: Option<u64>,
        status: &agave_geyser_plugin_interface::geyser_plugin_interface::SlotStatus,
        bank_id: solana_clock::BankId,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        if self.config.enable_bank_status {
            self.dispatch_payload(GeyserPayload::BankLifecycle {
                slot,
                bank_id,
                parent_slot: parent,
                status: status.as_str().to_string(),
                bank_hash: None,
            });
        }
        Ok(())
    }

    fn notify_block_metadata_for_bank(
        &self,
        blockinfo: agave_geyser_plugin_interface::geyser_plugin_interface::ReplicaBlockInfoVersions,
        _bank_id: solana_clock::BankId,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        match blockinfo {
            agave_geyser_plugin_interface::geyser_plugin_interface::ReplicaBlockInfoVersions::V0_0_1(b) => {
                self.dispatch_payload(GeyserPayload::BlockMeta {
                    slot: b.slot,
                    blockhash: b.blockhash.to_string(),
                    parent_slot: 0,
                    parent_blockhash: "".to_string(),
                    executed_tx_count: 0,
                });
            }
            agave_geyser_plugin_interface::geyser_plugin_interface::ReplicaBlockInfoVersions::V0_0_2(b) => {
                self.dispatch_payload(GeyserPayload::BlockMeta {
                    slot: b.slot,
                    blockhash: b.blockhash.to_string(),
                    parent_slot: b.parent_slot,
                    parent_blockhash: b.parent_blockhash.to_string(),
                    executed_tx_count: b.executed_transaction_count,
                });
            }
            agave_geyser_plugin_interface::geyser_plugin_interface::ReplicaBlockInfoVersions::V0_0_3(b) => {
                self.dispatch_payload(GeyserPayload::BlockMeta {
                    slot: b.slot,
                    blockhash: b.blockhash.to_string(),
                    parent_slot: b.parent_slot,
                    parent_blockhash: b.parent_blockhash.to_string(),
                    executed_tx_count: b.executed_transaction_count,
                });
            }
            agave_geyser_plugin_interface::geyser_plugin_interface::ReplicaBlockInfoVersions::V0_0_4(b) => {
                self.dispatch_payload(GeyserPayload::BlockMeta {
                    slot: b.slot,
                    blockhash: b.blockhash.to_string(),
                    parent_slot: b.parent_slot,
                    parent_blockhash: b.parent_blockhash.to_string(),
                    executed_tx_count: b.executed_transaction_count,
                });
            }
        }
        Ok(())
    }

    fn notify_entry_for_bank(
        &self,
        entry: agave_geyser_plugin_interface::geyser_plugin_interface::ReplicaEntryInfoVersions,
        bank_id: solana_clock::BankId,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        if self.config.enable_entries {
            match entry {
                agave_geyser_plugin_interface::geyser_plugin_interface::ReplicaEntryInfoVersions::V0_0_1(e) => {
                    self.dispatch_payload(GeyserPayload::Entry {
                        slot: e.slot,
                        bank_id: Some(bank_id),
                        entry_index: e.index as u64,
                        tx_count: e.executed_transaction_count,
                        starting_tx_index: 0,
                    });
                }
                agave_geyser_plugin_interface::geyser_plugin_interface::ReplicaEntryInfoVersions::V0_0_2(e) => {
                    self.dispatch_payload(GeyserPayload::Entry {
                        slot: e.slot,
                        bank_id: Some(bank_id),
                        entry_index: e.index as u64,
                        tx_count: e.executed_transaction_count,
                        starting_tx_index: e.starting_transaction_index as u64,
                    });
                }
            }
        }
        Ok(())
    }

    fn notify_entry_update_parent(
        &self,
        update_parent: agave_geyser_plugin_interface::geyser_plugin_interface::ReplicaEntryUpdateParentInfoVersions,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        if self.config.enable_update_parent {
            match update_parent {
                agave_geyser_plugin_interface::geyser_plugin_interface::ReplicaEntryUpdateParentInfoVersions::V0_0_1(u) => {
                    self.dispatch_payload(GeyserPayload::UpdateParent {
                        slot: u.slot,
                        cleared_bank_id: Some(u.cleared_bank_id),
                        parent_slot: u.parent_slot,
                        parent_block_id: Some(u.parent_block_id.to_string()),
                        fec_set_index: None,
                        source: "entry".to_string(),
                    });
                }
            }
        }
        Ok(())
    }

    fn notify_deshred_update_parent(
        &self,
        update_parent: agave_geyser_plugin_interface::geyser_plugin_interface::ReplicaDeshredUpdateParentInfoVersions,
    ) -> agave_geyser_plugin_interface::geyser_plugin_interface::Result<()> {
        if self.config.enable_update_parent {
            match update_parent {
                agave_geyser_plugin_interface::geyser_plugin_interface::ReplicaDeshredUpdateParentInfoVersions::V0_0_1(u) => {
                    self.dispatch_payload(GeyserPayload::UpdateParent {
                        slot: u.slot,
                        cleared_bank_id: None,
                        parent_slot: u.parent_slot,
                        parent_block_id: Some(u.parent_block_id.to_string()),
                        fec_set_index: Some(u.update_parent_fec_set_index),
                        source: "deshred".to_string(),
                    });
                }
            }
        }
        Ok(())
    }

    fn account_data_notifications_enabled(&self) -> bool {
        false
    }

    fn account_data_snapshot_notifications_enabled(&self) -> bool {
        false
    }

    fn transaction_notifications_enabled(&self) -> bool {
        false
    }

    fn entry_notifications_enabled(&self) -> bool {
        self.config.enable_entries
    }

    fn block_footer_notifications_enabled(&self) -> bool {
        self.config.enable_block_footer
    }

    fn deshred_transaction_notifications_enabled(&self) -> bool {
        self.config.enable_deshred
    }
}

