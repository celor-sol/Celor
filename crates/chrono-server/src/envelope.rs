use chrono_core::events::{ChronoEvent, ChronoEventKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventProvenance {
    DIRECT,
    DERIVED,
    ESTIMATED,
    UNAVAILABLE,
}

/// Versioned, typed Chrono Event Envelope emitted by the local Chrono service.
///
/// Features:
/// - Monotonically increasing service-local `sequence` (independent of Solana slot).
/// - Exact provenance (`DIRECT`, `DERIVED`, `ESTIMATED`, `UNAVAILABLE`).
/// - Strict separation of cluster ("testnet"), source ("rpc", "yellowstone"), and environment ("live", "local", "fixture").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronoServiceEvent {
    pub schema_version: u32,
    pub sequence: u64,
    pub event_id: u64,
    pub cluster: String,
    pub source: String,
    pub environment: String,
    pub observed_at_ms: u64,
    pub received_at_ms: u64,
    pub slot: u64,
    pub bank_id: Option<u64>,
    pub blockhash: Option<String>,
    pub parent_slot: Option<u64>,
    pub parent_blockhash: Option<String>,
    pub provenance: EventProvenance,
    pub event_type: String,
    pub payload: serde_json::Value,
    #[serde(default)]
    pub observer: Option<chrono_core::types::ObserverContext>,
}

impl ChronoServiceEvent {
    /// Creates a normalized service event envelope from a Chrono Core event.
    pub fn from_core_event(
        sequence: u64,
        cluster: &str,
        source: &str,
        environment: &str,
        event: &ChronoEvent,
    ) -> Self {
        let (event_type, bank_id, parent_slot, parent_blockhash, provenance, payload) = match &event.kind {
            ChronoEventKind::SlotObserved { parent_slot } => (
                "SlotObserved".to_string(),
                None,
                parent_slot.map(|s| s.as_u64()),
                None,
                EventProvenance::DIRECT,
                serde_json::json!({
                    "slot": event.slot.as_u64(),
                    "parent_slot": parent_slot.map(|s| s.as_u64()),
                }),
            ),
            ChronoEventKind::SlotStatusChanged { slot, status, bank_id } => (
                "SlotStatusChanged".to_string(),
                bank_id.as_ref().map(|b| b.0),
                None,
                None,
                EventProvenance::DIRECT,
                serde_json::json!({
                    "slot": slot.as_u64(),
                    "status": status,
                    "bank_id": bank_id.as_ref().map(|b| b.0),
                }),
            ),
            ChronoEventKind::LeaderObserved { leader } => (
                "LeaderObserved".to_string(),
                None,
                None,
                None,
                EventProvenance::DERIVED,
                serde_json::json!({
                    "slot": event.slot.as_u64(),
                    "leader": leader.as_str(),
                }),
            ),
            ChronoEventKind::LeaderScheduleObserved { starting_slot, leaders } => (
                "LeaderScheduleObserved".to_string(),
                None,
                None,
                None,
                EventProvenance::DERIVED,
                serde_json::json!({
                    "starting_slot": starting_slot.as_u64(),
                    "count": leaders.len(),
                    "sample_leader": leaders.first().map(|l| l.as_str()).unwrap_or(""),
                }),
            ),
            ChronoEventKind::BankCreated { identity, parent } => (
                "BankCreated".to_string(),
                identity.bank_id.as_ref().map(|b| b.0),
                None,
                None,
                if environment == "fixture" { EventProvenance::DERIVED } else { EventProvenance::DIRECT },
                serde_json::json!({
                    "bank_id": identity.bank_id.as_ref().map(|b| b.0),
                    "slot": identity.slot.as_u64(),
                    "parent": format!("{:?}", parent),
                }),
            ),
            ChronoEventKind::BankObserved { identity, parent, state } => (
                "BankObserved".to_string(),
                identity.bank_id.as_ref().map(|b| b.0),
                None,
                None,
                if environment == "fixture" { EventProvenance::DERIVED } else { EventProvenance::DIRECT },
                serde_json::json!({
                    "bank_id": identity.bank_id.as_ref().map(|b| b.0),
                    "slot": identity.slot.as_u64(),
                    "parent": format!("{:?}", parent),
                    "state": format!("{:?}", state),
                }),
            ),
            ChronoEventKind::BankStatusChanged { identity, status } => (
                "BankStatusChanged".to_string(),
                identity.bank_id.as_ref().map(|b| b.0),
                None,
                None,
                if environment == "fixture" { EventProvenance::DERIVED } else { EventProvenance::DIRECT },
                serde_json::json!({
                    "bank_id": identity.bank_id.as_ref().map(|b| b.0),
                    "status": format!("{:?}", status),
                }),
            ),
            ChronoEventKind::UpdateParent {
                slot,
                cleared_bank_id,
                parent_slot,
                parent_block_id,
                fec_set_index,
                source,
            } => (
                "UpdateParent".to_string(),
                cleared_bank_id.as_ref().map(|b| b.0),
                Some(parent_slot.as_u64()),
                parent_block_id.clone(),
                if environment == "fixture" { EventProvenance::DERIVED } else { EventProvenance::DIRECT },
                serde_json::json!({
                    "slot": slot.as_u64(),
                    "cleared_bank_id": cleared_bank_id.as_ref().map(|b| b.0),
                    "parent_slot": parent_slot.as_u64(),
                    "parent_block_id": parent_block_id,
                    "fec_set_index": fec_set_index,
                    "source": source,
                }),
            ),
            ChronoEventKind::ReplacementBank {
                old_identity,
                replacement_identity,
                slot,
            } => (
                "ReplacementBank".to_string(),
                replacement_identity.bank_id.as_ref().map(|b| b.0),
                None,
                None,
                EventProvenance::DERIVED,
                serde_json::json!({
                    "slot": slot.as_u64(),
                    "old_bank_id": old_identity.bank_id.as_ref().map(|b| b.0),
                    "replacement_bank_id": replacement_identity.bank_id.as_ref().map(|b| b.0),
                }),
            ),
            ChronoEventKind::BlockObserved {
                blockhash,
                parent_blockhash,
                tx_count,
            } => (
                "BlockObserved".to_string(),
                None,
                None,
                parent_blockhash.as_ref().map(|h| h.0.clone()),
                EventProvenance::DIRECT,
                serde_json::json!({
                    "slot": event.slot.as_u64(),
                    "blockhash": blockhash.0,
                    "parent_blockhash": parent_blockhash.as_ref().map(|h| &h.0),
                    "tx_count": tx_count,
                }),
            ),
            ChronoEventKind::BlockFooterObserved { footer } => (
                "BlockFooterObserved".to_string(),
                footer.bank_id.as_ref().map(|b| b.0),
                None,
                None,
                if environment == "fixture" { EventProvenance::DERIVED } else { EventProvenance::DIRECT },
                serde_json::json!({
                    "slot": footer.slot.as_u64(),
                    "bank_id": footer.bank_id.as_ref().map(|b| b.0),
                    "bank_hash": footer.bank_hash.as_ref().map(|h| &h.0),
                    "producer_time_nanos": footer.block_producer_time_nanos,
                    "user_agent": footer.block_user_agent,
                    "has_final_cert": footer.block_final_cert.is_some(),
                    "has_notar_cert": footer.notar_reward_cert.is_some(),
                }),
            ),
            ChronoEventKind::CertificateObserved {
                slot,
                kind,
                raw_len,
                raw_bytes,
                block_id,
                validation_status,
                stake_percent,
            } => (
                "CertificateObserved".to_string(),
                None,
                None,
                None,
                if environment == "fixture" { EventProvenance::DERIVED } else { EventProvenance::DIRECT },
                serde_json::json!({
                    "slot": slot.as_u64(),
                    "kind": format!("{:?}", kind),
                    "raw_len": raw_len,
                    "has_raw_bytes": raw_bytes.is_some(),
                    "block_id": block_id,
                    "validation_status": validation_status.map(|s| format!("{}", s)),
                    "stake_percent": stake_percent,
                }),
            ),
            ChronoEventKind::CanonicalObserved { canonical_identity } => (
                "CanonicalChanged".to_string(),
                canonical_identity.bank_id.as_ref().map(|b| b.0),
                None,
                None,
                EventProvenance::DERIVED,
                serde_json::json!({
                    "slot": canonical_identity.slot.as_u64(),
                    "bank_id": canonical_identity.bank_id.as_ref().map(|b| b.0),
                    "blockhash": canonical_identity.blockhash.as_ref().map(|h| &h.0),
                }),
            ),
            ChronoEventKind::FinalizedObserved {
                identity,
                cert_type,
                latency_ms,
            } => (
                "FinalityChanged".to_string(),
                identity.bank_id.as_ref().map(|b| b.0),
                None,
                None,
                if environment == "fixture" { EventProvenance::DERIVED } else { EventProvenance::DIRECT },
                serde_json::json!({
                    "slot": identity.slot.as_u64(),
                    "bank_id": identity.bank_id.as_ref().map(|b| b.0),
                    "cert_type": cert_type,
                    "latency_ms": latency_ms,
                }),
            ),
            ChronoEventKind::DeshredObserved {
                slot,
                signature,
                raw_tx_bytes,
                pre_execution_timestamp_nanos,
                static_accounts,
                fec_set_index,
            } => (
                "DeshredObserved".to_string(),
                None,
                None,
                None,
                if environment == "fixture" { EventProvenance::DERIVED } else { EventProvenance::DIRECT },
                serde_json::json!({
                    "slot": slot.as_u64(),
                    "signature": signature,
                    "raw_tx_len": raw_tx_bytes.as_ref().map(|b| b.len()),
                    "pre_execution_timestamp_nanos": pre_execution_timestamp_nanos,
                    "static_accounts": static_accounts,
                    "fec_set_index": fec_set_index,
                }),
            ),
            ChronoEventKind::ProducerTimingObserved {
                slot,
                producer_time_nanos,
                chrono_received_at_nanos,
                interval_nanos,
            } => (
                "ProducerTimingObserved".to_string(),
                None,
                None,
                None,
                if environment == "fixture" { EventProvenance::DERIVED } else { EventProvenance::DIRECT },
                serde_json::json!({
                    "slot": slot.as_u64(),
                    "producer_time_nanos": producer_time_nanos,
                    "chrono_received_at_nanos": chrono_received_at_nanos,
                    "interval_nanos": interval_nanos,
                    "interval_ms": (*interval_nanos as f64) / 1_000_000.0,
                }),
            ),
            ChronoEventKind::ProviderConnected { provider } => (
                "StreamConnected".to_string(),
                None,
                None,
                None,
                EventProvenance::DIRECT,
                serde_json::json!({
                    "provider": provider.as_str(),
                }),
            ),
            ChronoEventKind::ProviderDisconnected { provider, reason } => (
                "StreamDisconnected".to_string(),
                None,
                None,
                None,
                EventProvenance::DIRECT,
                serde_json::json!({
                    "provider": provider.as_str(),
                    "reason": reason,
                }),
            ),
            other => (
                format!("{:?}", other),
                None,
                None,
                None,
                EventProvenance::DERIVED,
                serde_json::json!({}),
            ),
        };

        Self {
            schema_version: 1,
            sequence,
            event_id: event.event_id,
            cluster: cluster.to_string(),
            source: source.to_string(),
            environment: environment.to_string(),
            observed_at_ms: event.received_time_ms,
            received_at_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            slot: event.slot.as_u64(),
            bank_id,
            blockhash: event.blockhash.as_ref().map(|h| h.0.clone()),
            parent_slot,
            parent_blockhash,
            provenance,
            event_type,
            payload,
            observer: event.observer.clone(),
        }
    }
}
