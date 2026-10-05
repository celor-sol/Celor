use crate::events::ChronoGeyserRawFrame;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TrySendError};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// High-performance non-blocking IPC client for shipping Geyser callbacks
/// from validator memory space to Chrono Core over Unix Domain Sockets.
#[derive(Clone, Debug)]
pub struct GeyserIpcWriter {
    sender: SyncSender<ChronoGeyserRawFrame>,
    sent_count: Arc<AtomicU64>,
    dropped_count: Arc<AtomicU64>,
    connected: Arc<AtomicBool>,
}

impl GeyserIpcWriter {
    pub fn new(socket_path: String, capacity: usize) -> Self {
        let (tx, rx) = sync_channel::<ChronoGeyserRawFrame>(capacity);
        let sent_count = Arc::new(AtomicU64::new(0));
        let dropped_count = Arc::new(AtomicU64::new(0));
        let connected = Arc::new(AtomicBool::new(false));

        let worker_sent = sent_count.clone();
        let worker_conn = connected.clone();

        thread::spawn(move || {
            Self::worker_loop(socket_path, rx, worker_sent, worker_conn);
        });

        Self {
            sender: tx,
            sent_count,
            dropped_count,
            connected,
        }
    }

    /// Dispatches an event without ever blocking the validator thread.
    /// Returns true if queued, false if dropped due to buffer capacity.
    pub fn dispatch(&self, frame: ChronoGeyserRawFrame) -> bool {
        match self.sender.try_send(frame) {
            Ok(()) => true,
            Err(TrySendError::Full(_)) => {
                self.dropped_count.fetch_add(1, Ordering::Relaxed);
                false
            }
            Err(TrySendError::Disconnected(_)) => {
                self.dropped_count.fetch_add(1, Ordering::Relaxed);
                false
            }
        }
    }

    pub fn sent_count(&self) -> u64 {
        self.sent_count.load(Ordering::Relaxed)
    }

    pub fn dropped_count(&self) -> u64 {
        self.dropped_count.load(Ordering::Relaxed)
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    fn worker_loop(
        socket_path: String,
        receiver: Receiver<ChronoGeyserRawFrame>,
        sent_count: Arc<AtomicU64>,
        connected: Arc<AtomicBool>,
    ) {
        let mut stream_opt: Option<UnixStream> = None;

        while let Ok(frame) = receiver.recv() {
            // Ensure connection
            if stream_opt.is_none() {
                match UnixStream::connect(&socket_path) {
                    Ok(s) => {
                        info!("Chrono Geyser Plugin connected to UDS at {}", socket_path);
                        let _ = s.set_nonblocking(false);
                        stream_opt = Some(s);
                        connected.store(true, Ordering::SeqCst);
                    }
                    Err(e) => {
                        debug!("Chrono Geyser waiting for UDS listener {}: {}", socket_path, e);
                        connected.store(false, Ordering::SeqCst);
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                }
            }

            if let Some(ref mut stream) = stream_opt {
                match serde_json::to_vec(&frame) {
                    Ok(mut bytes) => {
                        bytes.push(b'\n'); // NDJSON line delimiter
                        if let Err(e) = stream.write_all(&bytes) {
                            warn!("Write error to Chrono UDS: {}. Reconnecting...", e);
                            stream_opt = None;
                            connected.store(false, Ordering::SeqCst);
                        } else {
                            sent_count.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    Err(e) => {
                        error!("Failed to serialize Geyser frame: {}", e);
                    }
                }
            }
        }
    }
}
