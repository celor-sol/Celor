use crate::provider::AdapterError;
use chrono_core::types::{LeaderId, Slot};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochInfo {
    pub absolute_slot: u64,
    pub block_height: u64,
    pub epoch: u64,
    pub slot_index: u64,
    pub slots_in_epoch: u64,
    pub transaction_count: Option<u64>,
}

/// Robust HTTP JSON-RPC Client for Solana Clusters.
pub struct SolanaRpcClient {
    client: Client,
    rpc_url: String,
}

impl SolanaRpcClient {
    pub fn new(rpc_url: impl Into<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(6))
                .build()
                .unwrap_or_default(),
            rpc_url: rpc_url.into(),
        }
    }

    pub fn rpc_url(&self) -> &str {
        &self.rpc_url
    }

    pub async fn get_slot(&self) -> Result<Slot, AdapterError> {
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getSlot",
            "params": [{"commitment": "processed"}]
        });

        let resp = self.send_rpc(&payload).await?;
        let slot = resp
            .as_u64()
            .ok_or_else(|| AdapterError::InvalidPayload("Expected u64 slot".into()))?;

        Ok(Slot(slot))
    }

    pub async fn get_slot_leader(&self) -> Result<LeaderId, AdapterError> {
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getSlotLeader"
        });

        let resp = self.send_rpc(&payload).await?;
        let leader_str = resp
            .as_str()
            .ok_or_else(|| AdapterError::InvalidPayload("Expected leader string".into()))?;

        Ok(LeaderId::new(leader_str))
    }

    /// Fetches upcoming slot leaders starting from `start_slot` up to `limit`.
    pub async fn get_slot_leaders(&self, start_slot: u64, limit: u64) -> Result<Vec<LeaderId>, AdapterError> {
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getSlotLeaders",
            "params": [start_slot, limit]
        });

        let resp = self.send_rpc(&payload).await?;
        let array = resp
            .as_array()
            .ok_or_else(|| AdapterError::InvalidPayload("Expected array of leaders".into()))?;

        let leaders = array
            .iter()
            .filter_map(|v| v.as_str().map(LeaderId::new))
            .collect();

        Ok(leaders)
    }

    pub async fn get_genesis_hash(&self) -> Result<String, AdapterError> {
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getGenesisHash"
        });

        let resp = self.send_rpc(&payload).await?;
        let hash = resp
            .as_str()
            .ok_or_else(|| AdapterError::InvalidPayload("Expected genesis hash".into()))?;

        Ok(hash.to_string())
    }

    pub async fn get_epoch_info(&self) -> Result<EpochInfo, AdapterError> {
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getEpochInfo",
            "params": [{"commitment": "processed"}]
        });

        let resp = self.send_rpc(&payload).await?;
        let epoch = resp
            .get("epoch")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let absolute_slot = resp
            .get("absoluteSlot")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let block_height = resp
            .get("blockHeight")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let slot_index = resp
            .get("slotIndex")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let slots_in_epoch = resp
            .get("slotsInEpoch")
            .and_then(|v| v.as_u64())
            .unwrap_or(432000);
        let transaction_count = resp
            .get("transactionCount")
            .and_then(|v| v.as_u64());

        Ok(EpochInfo {
            absolute_slot,
            block_height,
            epoch,
            slot_index,
            slots_in_epoch,
            transaction_count,
        })
    }

    /// Fetches leader schedule and returns a mapping from slot to leader.
    pub async fn get_leader_schedule(&self, slot: Slot) -> Result<HashMap<Slot, LeaderId>, AdapterError> {
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getLeaderSchedule",
            "params": [slot.as_u64(), {"commitment": "processed"}]
        });

        let resp = self.send_rpc(&payload).await?;
        let mut schedule = HashMap::new();

        if let Some(map) = resp.as_object() {
            for (leader_pubkey, slots) in map {
                if let Some(slot_indices) = slots.as_array() {
                    for idx in slot_indices {
                        if let Some(i) = idx.as_u64() {
                            // Map index to absolute slot in epoch window
                            schedule.insert(Slot(slot.as_u64() + i), LeaderId::new(leader_pubkey));
                        }
                    }
                }
            }
        }

        Ok(schedule)
    }

    async fn send_rpc(&self, payload: &serde_json::Value) -> Result<serde_json::Value, AdapterError> {
        let resp = self
            .client
            .post(&self.rpc_url)
            .json(payload)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        if let Some(err) = resp.get("error") {
            return Err(AdapterError::JsonRpc(err.to_string()));
        }

        resp.get("result")
            .cloned()
            .ok_or_else(|| AdapterError::InvalidPayload("Missing result field in RPC response".into()))
    }
}
