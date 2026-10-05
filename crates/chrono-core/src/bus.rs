use crate::events::ChronoEvent;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;

/// High-throughput, non-blocking internal Event Bus.
///
/// Designed so that slow consumers cannot stall ingestion.
/// Uses bounded ring buffer broadcast channels.
#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<Arc<ChronoEvent>>,
    event_counter: Arc<AtomicU64>,
}

impl EventBus {
    pub const DEFAULT_CAPACITY: usize = 4096;

    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender,
            event_counter: Arc::new(AtomicU64::new(1)),
        }
    }

    /// Next unique sequential event sequence number.
    pub fn next_event_id(&self) -> u64 {
        self.event_counter.fetch_add(1, Ordering::Relaxed)
    }

    /// Publish an event to all active subscribers. Non-blocking.
    pub fn publish(&self, event: ChronoEvent) -> usize {
        let arc_event = Arc::new(event);
        // send() returns the number of active receivers
        self.sender.send(arc_event).unwrap_or(0)
    }

    /// Subscribe to the event stream. Returns a broadcast Receiver.
    pub fn subscribe(&self) -> broadcast::Receiver<Arc<ChronoEvent>> {
        self.sender.subscribe()
    }

    /// Current receiver count.
    pub fn receiver_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(Self::DEFAULT_CAPACITY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::ChronoEventKind;
    use crate::types::{ProviderId, Slot};

    #[tokio::test]
    async fn test_event_bus_publish_subscribe() {
        let bus = EventBus::new(32);
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        let event = ChronoEvent::new(
            bus.next_event_id(),
            ProviderId::new("test"),
            Slot(100),
            None,
            ChronoEventKind::SlotObserved { parent_slot: None },
        );

        let count = bus.publish(event);
        assert_eq!(count, 2);

        let received1 = rx1.recv().await.unwrap();
        assert_eq!(received1.slot, Slot(100));

        let received2 = rx2.recv().await.unwrap();
        assert_eq!(received2.slot, Slot(100));
    }
}
