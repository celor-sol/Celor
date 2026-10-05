use chrono_core::types::FieldProvenance;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Capability state of a single Alpenglow telemetry field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldSupport {
    Supported,
    DerivedOnly,
    Unsupported,
    Unknown,
}

impl fmt::Display for FieldSupport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Supported => write!(f, "YES"),
            Self::DerivedOnly => write!(f, "DERIVED"),
            Self::Unsupported => write!(f, "NO"),
            Self::Unknown => write!(f, "?"),
        }
    }
}

/// Detailed runtime capabilities of an ingestion stream or provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderCapabilityMatrix {
    pub provider_name: String,
    pub slot: FieldSupport,
    pub leader: FieldSupport,
    pub bank_id: FieldSupport,
    pub bank_hash: FieldSupport,
    pub parent: FieldSupport,
    pub update_parent: FieldSupport,
    pub block_footer: FieldSupport,
    pub certificates: FieldSupport,
    pub producer_time: FieldSupport,
    pub producer_user_agent: FieldSupport,
    pub deshred: FieldSupport,
}

impl ProviderCapabilityMatrix {
    /// Capabilities for standard public Solana JSON-RPC / WebSocket.
    pub fn standard_public_rpc(provider_name: impl Into<String>) -> Self {
        Self {
            provider_name: provider_name.into(),
            slot: FieldSupport::Supported,
            leader: FieldSupport::DerivedOnly, // Derived from getSlotLeaders or schedule
            bank_id: FieldSupport::Unsupported, // Standard public RPC does not expose bank_id
            bank_hash: FieldSupport::Unsupported,
            parent: FieldSupport::Supported,
            update_parent: FieldSupport::Unsupported,
            block_footer: FieldSupport::Unsupported,
            certificates: FieldSupport::Unsupported,
            producer_time: FieldSupport::Unsupported,
            producer_user_agent: FieldSupport::Unsupported,
            deshred: FieldSupport::Unsupported,
        }
    }

    /// Capabilities for full-fidelity Yellowstone gRPC / Geyser.
    pub fn yellowstone_grpc(provider_name: impl Into<String>) -> Self {
        Self {
            provider_name: provider_name.into(),
            slot: FieldSupport::Supported,
            leader: FieldSupport::Supported,
            bank_id: FieldSupport::Supported,
            bank_hash: FieldSupport::Supported,
            parent: FieldSupport::Supported,
            update_parent: FieldSupport::Supported,
            block_footer: FieldSupport::Supported,
            certificates: FieldSupport::Supported,
            producer_time: FieldSupport::Supported,
            producer_user_agent: FieldSupport::Supported,
            deshred: FieldSupport::Supported,
        }
    }

    /// Capabilities for Local Validator + Geyser Plugin.
    pub fn local_geyser(provider_name: impl Into<String>) -> Self {
        Self {
            provider_name: provider_name.into(),
            slot: FieldSupport::Supported,
            leader: FieldSupport::Supported,
            bank_id: FieldSupport::Supported,
            bank_hash: FieldSupport::Supported,
            parent: FieldSupport::Supported,
            update_parent: FieldSupport::Supported,
            block_footer: FieldSupport::Supported,
            certificates: FieldSupport::Supported,
            producer_time: FieldSupport::Supported,
            producer_user_agent: FieldSupport::Supported,
            deshred: FieldSupport::Supported,
        }
    }

    /// Calculates the Alpenglow Coverage Score (0% to 100%).
    ///
    /// Evaluates 10 core Alpenglow architectural dimensions:
    /// - Slot
    /// - Leader
    /// - Bank ID
    /// - Bank Hash
    /// - Parent Lineage
    /// - UpdateParent
    /// - Block Footer
    /// - Certificates (BLS Fast-Path / Fallback)
    /// - Producer Time
    /// - Producer User Agent
    pub fn coverage_score(&self) -> u32 {
        let mut score = 0;
        let checks = [
            self.slot,
            self.leader,
            self.bank_id,
            self.bank_hash,
            self.parent,
            self.update_parent,
            self.block_footer,
            self.certificates,
            self.producer_time,
            self.producer_user_agent,
        ];

        for c in checks {
            match c {
                FieldSupport::Supported => score += 10,
                FieldSupport::DerivedOnly => score += 7,
                FieldSupport::Unknown => score += 2,
                FieldSupport::Unsupported => {}
            }
        }
        score
    }

    /// Capabilities for a Real Local Agave Validator with Chrono Geyser Plugin over UDS.
    pub fn local_validator(provider_name: impl Into<String>) -> Self {
        Self {
            provider_name: provider_name.into(),
            slot: FieldSupport::Supported,
            leader: FieldSupport::Supported,
            bank_id: FieldSupport::Supported,
            bank_hash: FieldSupport::Supported,
            parent: FieldSupport::Supported,
            update_parent: FieldSupport::Supported,
            block_footer: FieldSupport::Supported,
            certificates: FieldSupport::Supported,
            producer_time: FieldSupport::Supported,
            producer_user_agent: FieldSupport::Supported,
            deshred: FieldSupport::Supported,
        }
    }

    /// Capabilities for Local Geyser Specification Fixture simulation.
    pub fn local_geyser_fixture(provider_name: impl Into<String>) -> Self {
        Self::local_geyser(provider_name)
    }

    /// Generates exhaustive TelemetryCapabilityMatrix with detailed diagnostics per field.
    pub fn to_detailed_matrix(
        &self,
        observer: chrono_core::types::ObserverContext,
        is_live_observed: bool,
    ) -> chrono_core::types::TelemetryCapabilityMatrix {
        let is_fixture = self.provider_name.contains("fixture");
        let is_rpc = self.provider_name.contains("rpc") || self.provider_name == "testnet" || self.provider_name == "devnet" || self.provider_name == "mainnet";
        let is_geyser = self.provider_name.contains("validator") || self.provider_name.contains("geyser");

        let level = if is_rpc {
            chrono_core::types::TelemetryLevel::Level0PublicRpc
        } else if self.provider_name.contains("yellowstone") {
            chrono_core::types::TelemetryLevel::Level2Yellowstone
        } else if is_geyser && !is_fixture {
            chrono_core::types::TelemetryLevel::Level4ChronoValidator
        } else {
            chrono_core::types::TelemetryLevel::Level3ValidatorGeyser
        };

        let fields = vec![
            chrono_core::types::FieldAvailabilityDetail {
                field: "slot".to_string(),
                protocol_supported: true,
                source_supported: self.slot == FieldSupport::Supported,
                currently_observed: is_live_observed,
                provenance: chrono_core::types::FieldProvenance::Direct,
                freshness: if is_live_observed { "LIVE".to_string() } else { "NOT_OBSERVED".to_string() },
                reason_unavailable: None,
                alternate_source: None,
            },
            chrono_core::types::FieldAvailabilityDetail {
                field: "leader".to_string(),
                protocol_supported: true,
                source_supported: true,
                currently_observed: is_live_observed,
                provenance: chrono_core::types::FieldProvenance::Derived,
                freshness: if is_live_observed { "LIVE".to_string() } else { "NOT_OBSERVED".to_string() },
                reason_unavailable: None,
                alternate_source: None,
            },
            chrono_core::types::FieldAvailabilityDetail {
                field: "bank_id".to_string(),
                protocol_supported: true,
                source_supported: self.bank_id == FieldSupport::Supported,
                currently_observed: is_live_observed && self.bank_id == FieldSupport::Supported,
                provenance: if self.bank_id == FieldSupport::Supported { chrono_core::types::FieldProvenance::Direct } else { chrono_core::types::FieldProvenance::Unavailable },
                freshness: if is_live_observed && self.bank_id == FieldSupport::Supported { "LIVE".to_string() } else { "UNAVAILABLE".to_string() },
                reason_unavailable: if self.bank_id != FieldSupport::Supported {
                    Some("Validator-local candidate bank identity; standard Solana JSON-RPC does not expose bank_id (requires Yellowstone gRPC or Geyser plugin)".to_string())
                } else {
                    None
                },
                alternate_source: Some("Agave Geyser Plugin / Yellowstone gRPC".to_string()),
            },
            chrono_core::types::FieldAvailabilityDetail {
                field: "bank_hash".to_string(),
                protocol_supported: true,
                source_supported: self.bank_hash == FieldSupport::Supported,
                currently_observed: is_live_observed && self.bank_hash == FieldSupport::Supported,
                provenance: if self.bank_hash == FieldSupport::Supported { chrono_core::types::FieldProvenance::Direct } else { chrono_core::types::FieldProvenance::Unavailable },
                freshness: if is_live_observed && self.bank_hash == FieldSupport::Supported { "LIVE".to_string() } else { "UNAVAILABLE".to_string() },
                reason_unavailable: if self.bank_hash != FieldSupport::Supported {
                    Some("Interim accounts delta state accumulator; omitted by standard public RPC (requires Yellowstone gRPC or Geyser plugin)".to_string())
                } else {
                    None
                },
                alternate_source: Some("Block Footer / Geyser Plugin".to_string()),
            },
            chrono_core::types::FieldAvailabilityDetail {
                field: "parent_slot".to_string(),
                protocol_supported: true,
                source_supported: self.parent == FieldSupport::Supported,
                currently_observed: is_live_observed,
                provenance: chrono_core::types::FieldProvenance::Direct,
                freshness: if is_live_observed { "LIVE".to_string() } else { "NOT_OBSERVED".to_string() },
                reason_unavailable: None,
                alternate_source: None,
            },
            chrono_core::types::FieldAvailabilityDetail {
                field: "update_parent".to_string(),
                protocol_supported: true,
                source_supported: self.update_parent == FieldSupport::Supported,
                currently_observed: is_live_observed && self.update_parent == FieldSupport::Supported,
                provenance: if self.update_parent == FieldSupport::Supported { chrono_core::types::FieldProvenance::Direct } else { chrono_core::types::FieldProvenance::Unavailable },
                freshness: if is_live_observed && self.update_parent == FieldSupport::Supported { "LIVE".to_string() } else { "UNAVAILABLE".to_string() },
                reason_unavailable: if self.update_parent != FieldSupport::Supported {
                    Some("Fast leader handover marker; omitted by standard public RPC (requires Yellowstone gRPC or Geyser plugin)".to_string())
                } else {
                    None
                },
                alternate_source: Some("Agave Geyser notify_entry_update_parent / Yellowstone SubscribeUpdateEntry".to_string()),
            },
            chrono_core::types::FieldAvailabilityDetail {
                field: "block_footer".to_string(),
                protocol_supported: true,
                source_supported: self.block_footer == FieldSupport::Supported,
                currently_observed: is_live_observed && self.block_footer == FieldSupport::Supported,
                provenance: if self.block_footer == FieldSupport::Supported { chrono_core::types::FieldProvenance::Direct } else { chrono_core::types::FieldProvenance::Unavailable },
                freshness: if is_live_observed && self.block_footer == FieldSupport::Supported { "LIVE".to_string() } else { "UNAVAILABLE".to_string() },
                reason_unavailable: if self.block_footer != FieldSupport::Supported {
                    Some("Alpenglow block footer containing certificates and producer metadata; requires Yellowstone gRPC or Geyser plugin".to_string())
                } else {
                    None
                },
                alternate_source: Some("Agave Geyser notify_block_footer".to_string()),
            },
            chrono_core::types::FieldAvailabilityDetail {
                field: "certificates".to_string(),
                protocol_supported: true,
                source_supported: self.certificates == FieldSupport::Supported,
                currently_observed: is_live_observed && self.certificates == FieldSupport::Supported,
                provenance: if self.certificates == FieldSupport::Supported { chrono_core::types::FieldProvenance::Direct } else { chrono_core::types::FieldProvenance::Unavailable },
                freshness: if is_live_observed && self.certificates == FieldSupport::Supported { "LIVE".to_string() } else { "UNAVAILABLE".to_string() },
                reason_unavailable: if self.certificates != FieldSupport::Supported {
                    Some("Alpenglow BLS aggregate certificates; omitted by public RPC and stripped by standard Yellowstone streams".to_string())
                } else {
                    None
                },
                alternate_source: Some("Chrono Validator Geyser Plugin / Devnet RPC getAgGenesisCert".to_string()),
            },
            chrono_core::types::FieldAvailabilityDetail {
                field: "producer_time".to_string(),
                protocol_supported: true,
                source_supported: self.producer_time == FieldSupport::Supported,
                currently_observed: is_live_observed && self.producer_time == FieldSupport::Supported,
                provenance: if self.producer_time == FieldSupport::Supported { chrono_core::types::FieldProvenance::Direct } else { chrono_core::types::FieldProvenance::Unavailable },
                freshness: if is_live_observed && self.producer_time == FieldSupport::Supported { "LIVE".to_string() } else { "UNAVAILABLE".to_string() },
                reason_unavailable: if self.producer_time != FieldSupport::Supported {
                    Some("High-precision nanosecond producer timestamp; only emitted in Alpenglow block footer".to_string())
                } else {
                    None
                },
                alternate_source: Some("Block Footer / Geyser Plugin".to_string()),
            },
            chrono_core::types::FieldAvailabilityDetail {
                field: "producer_user_agent".to_string(),
                protocol_supported: true,
                source_supported: self.producer_user_agent == FieldSupport::Supported,
                currently_observed: is_live_observed && self.producer_user_agent == FieldSupport::Supported,
                provenance: if self.producer_user_agent == FieldSupport::Supported { chrono_core::types::FieldProvenance::Direct } else { chrono_core::types::FieldProvenance::Unavailable },
                freshness: if is_live_observed && self.producer_user_agent == FieldSupport::Supported { "LIVE".to_string() } else { "UNAVAILABLE".to_string() },
                reason_unavailable: if self.producer_user_agent != FieldSupport::Supported {
                    Some("Producing validator client identifier; emitted in block footer".to_string())
                } else {
                    None
                },
                alternate_source: Some("Block Footer / Geyser Plugin".to_string()),
            },
            chrono_core::types::FieldAvailabilityDetail {
                field: "deshred".to_string(),
                protocol_supported: true,
                source_supported: self.deshred == FieldSupport::Supported,
                currently_observed: is_live_observed && self.deshred == FieldSupport::Supported,
                provenance: if self.deshred == FieldSupport::Supported { chrono_core::types::FieldProvenance::Direct } else { chrono_core::types::FieldProvenance::Unavailable },
                freshness: if is_live_observed && self.deshred == FieldSupport::Supported { "LIVE".to_string() } else { "UNAVAILABLE".to_string() },
                reason_unavailable: if self.deshred != FieldSupport::Supported {
                    Some("Pre-execution transaction stream; open-source Yellowstone returns UNIMPLEMENTED (requires Chrono Geyser Plugin)".to_string())
                } else {
                    None
                },
                alternate_source: Some("Chrono Validator Geyser notify_deshred_transaction".to_string()),
            },
        ];

        let core_supported_count = fields.iter().take(10).filter(|f| f.source_supported).count();
        let core_pct = (core_supported_count as u32 * 10).min(100);

        let ext_supported_count = fields.iter().filter(|f| f.source_supported).count();
        let ext_pct = ((ext_supported_count as f64 / fields.len() as f64) * 100.0).round() as u32;

        chrono_core::types::TelemetryCapabilityMatrix {
            current_source: self.provider_name.clone(),
            telemetry_level: level,
            observer,
            fields,
            core_coverage_percent: if is_rpc { 27 } else { core_pct },
            extended_coverage_percent: if is_rpc { 20 } else { ext_pct },
        }
    }

    /// Returns field provenance for a requested field under this provider.
    pub fn field_provenance(&self, field: &str) -> FieldProvenance {
        match field {
            "slot" => match self.slot {
                FieldSupport::Supported => FieldProvenance::Direct,
                FieldSupport::DerivedOnly => FieldProvenance::Derived,
                _ => FieldProvenance::Unavailable,
            },
            "leader" => match self.leader {
                FieldSupport::Supported => FieldProvenance::Direct,
                FieldSupport::DerivedOnly => FieldProvenance::Derived,
                _ => FieldProvenance::Unavailable,
            },
            "bank_id" => match self.bank_id {
                FieldSupport::Supported => FieldProvenance::Direct,
                _ => FieldProvenance::Unavailable,
            },
            "bank_hash" => match self.bank_hash {
                FieldSupport::Supported => FieldProvenance::Direct,
                _ => FieldProvenance::Unavailable,
            },
            "update_parent" => match self.update_parent {
                FieldSupport::Supported => FieldProvenance::Direct,
                _ => FieldProvenance::Unavailable,
            },
            "certificates" => match self.certificates {
                FieldSupport::Supported => FieldProvenance::Direct,
                _ => FieldProvenance::Unavailable,
            },
            "producer_time" => match self.producer_time {
                FieldSupport::Supported => FieldProvenance::Direct,
                _ => FieldProvenance::Unavailable,
            },
            _ => FieldProvenance::Unavailable,
        }
    }
}
