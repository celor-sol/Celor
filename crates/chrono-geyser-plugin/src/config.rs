use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    #[serde(default)]
    pub libpath: Option<String>,
    #[serde(default = "default_socket_path")]
    pub uds_socket_path: String,
    #[serde(default = "default_observer_id")]
    pub observer_validator_id: String,
    #[serde(default = "default_true")]
    pub enable_bank_status: bool,
    #[serde(default = "default_true")]
    pub enable_block_footer: bool,
    #[serde(default = "default_true")]
    pub enable_update_parent: bool,
    #[serde(default = "default_true")]
    pub enable_deshred: bool,
    #[serde(default = "default_true")]
    pub enable_entries: bool,
    #[serde(default = "default_buffer_capacity")]
    pub channel_buffer_capacity: usize,
}

fn default_socket_path() -> String {
    "/tmp/chrono_geyser.sock".to_string()
}

fn default_observer_id() -> String {
    "local-agave-validator".to_string()
}

fn default_true() -> bool {
    true
}

fn default_buffer_capacity() -> usize {
    65536
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            libpath: None,
            uds_socket_path: default_socket_path(),
            observer_validator_id: default_observer_id(),
            enable_bank_status: true,
            enable_block_footer: true,
            enable_update_parent: true,
            enable_deshred: true,
            enable_entries: true,
            channel_buffer_capacity: default_buffer_capacity(),
        }
    }
}
