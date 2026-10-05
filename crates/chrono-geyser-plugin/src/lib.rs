pub mod config;
pub mod events;
pub mod ipc;
pub mod plugin;

pub use config::PluginConfig;
pub use events::{ChronoGeyserRawFrame, GeyserPayload};
pub use ipc::GeyserIpcWriter;
pub use plugin::ChronoGeyserPlugin;

/// Dynamic C ABI entry point resolved by Solana / Agave validator dynamic loader.
///
/// # Safety
/// The caller must ensure that the returned pointer is appropriately managed or dropped
/// via the GeyserPlugin lifecycle methods.
#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub unsafe extern "C" fn _create_plugin() -> *mut dyn agave_geyser_plugin_interface::geyser_plugin_interface::GeyserPlugin {
    let plugin = Box::new(ChronoGeyserPlugin::new());
    Box::into_raw(plugin)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader};
    use std::os::unix::net::UnixListener;
    use std::path::Path;
    use std::thread;

    #[test]
    fn test_geyser_ipc_framing_and_transmission() {
        let sock_path = format!("/tmp/test_chrono_geyser_{}.sock", std::process::id());
        if Path::new(&sock_path).exists() {
            let _ = std::fs::remove_file(&sock_path);
        }

        let listener = UnixListener::bind(&sock_path).expect("Failed to bind test UDS");

        let handle = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("Accept failed");
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).expect("Read line failed");
            let frame: ChronoGeyserRawFrame = serde_json::from_str(&line).expect("JSON parse failed");
            frame
        });

        // Initialize plugin with test socket
        let config = PluginConfig {
            uds_socket_path: sock_path.clone(),
            observer_validator_id: "test-node-1".to_string(),
            ..Default::default()
        };

        let plugin = ChronoGeyserPlugin::with_config(config);
        plugin.update_bank_status(100, Some(99), "Processed", 1, Some("test_bank_hash".to_string()));

        let received_frame = handle.join().expect("Worker thread panicked");
        assert_eq!(received_frame.observer_validator, "test-node-1");
        assert_eq!(received_frame.sequence, 1);
        match received_frame.payload {
            GeyserPayload::BankLifecycle { slot, bank_id, status, .. } => {
                assert_eq!(slot, 100);
                assert_eq!(bank_id, 1);
                assert_eq!(status, "Processed");
            }
            _ => panic!("Expected BankLifecycle payload"),
        }

        let _ = std::fs::remove_file(&sock_path);
    }
}
