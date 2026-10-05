use serde::{Deserialize, Serialize};

/// Alpenglow Genesis Certificate block data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgGenesisBlock {
    #[serde(rename = "blockId")]
    pub block_id: Vec<u8>,
    pub slot: u64,
}

/// Alpenglow BLS aggregate certificate signature and participation bitmap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgGenesisSignature {
    pub bitmap: Vec<u8>,
    pub signature: Vec<u8>,
}

/// Structure of a verified Alpenglow Genesis Certificate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgGenesisCert {
    pub block: AgGenesisBlock,
    pub signature: AgGenesisSignature,
}

/// Result of evaluating the cluster's `getAgGenesisCert` endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenesisCertResponse {
    /// Certificate was successfully returned and parsed.
    Certificate(AgGenesisCert),
    /// Method succeeded but returned null (Alpenglow not active on cluster, e.g. Mainnet-Beta).
    Null,
    /// Method is not implemented / MethodNotFound (-32601) on the validator node.
    MethodUnavailable,
    /// RPC or network error.
    Error(String),
}

impl GenesisCertResponse {
    pub fn is_alpenglow_active(&self) -> bool {
        matches!(self, Self::Certificate(_))
    }
}
