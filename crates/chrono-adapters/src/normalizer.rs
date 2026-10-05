use crate::ws_stream::RawSlotNotification;
use chrono_core::events::{ChronoEvent, ChronoEventKind};
use chrono_core::types::{ProviderId, Slot};
use std::collections::HashSet;

/// Stream Normalizer with Deduplication and Out-of-Order Convergence.
///
/// Ensures repeated messages (same event, same slot, same provider)
/// do not contaminate internal state or create duplicate transitions.
pub struct StreamNormalizer {
    provider_id: ProviderId,
    seen_slots: HashSet<Slot>,
    max_history: usize,
    event_sequence: u64,
}

impl StreamNormalizer {
    pub fn new(provider_id: ProviderId, max_history: usize) -> Self {
        Self {
            provider_id,
            seen_slots: HashSet::with_capacity(max_history),
            max_history,
            event_sequence: 0,
        }
    }

    /// Normalizes a raw slot notification into a ChronoEvent.
    /// Returns `None` if the event is a duplicate.
    pub fn normalize_slot(&mut self, raw: RawSlotNotification) -> Option<ChronoEvent> {
        // Deduplication check
        if self.seen_slots.contains(&raw.slot) {
            return None;
        }

        if self.seen_slots.len() >= self.max_history {
            // Prune slots below current - max_history
            let min_keep = raw.slot.as_u64().saturating_sub(self.max_history as u64);
            self.seen_slots.retain(|s| s.as_u64() >= min_keep);
        }

        self.seen_slots.insert(raw.slot);
        self.event_sequence += 1;

        Some(ChronoEvent::new(
            self.event_sequence,
            self.provider_id.clone(),
            raw.slot,
            None,
            ChronoEventKind::SlotObserved {
                parent_slot: raw.parent,
            },
        ))
    }

    pub fn seen_count(&self) -> usize {
        self.seen_slots.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicate_events_deduplicated() {
        let mut normalizer = StreamNormalizer::new(ProviderId::new("test-provider"), 100);

        let raw1 = RawSlotNotification {
            slot: Slot(500),
            parent: Some(Slot(499)),
            root: Some(Slot(468)),
        };

        let raw2 = RawSlotNotification {
            slot: Slot(500), // Duplicate slot!
            parent: Some(Slot(499)),
            root: Some(Slot(468)),
        };

        let ev1 = normalizer.normalize_slot(raw1);
        assert!(ev1.is_some());
        assert_eq!(ev1.unwrap().slot, Slot(500));

        // Second duplicate must return None!
        let ev2 = normalizer.normalize_slot(raw2);
        assert!(ev2.is_none());
    }

    #[test]
    fn test_out_of_order_slots_processed_safely() {
        let mut normalizer = StreamNormalizer::new(ProviderId::new("test-provider"), 100);

        // Slot 502 arrives first
        let ev1 = normalizer.normalize_slot(RawSlotNotification {
            slot: Slot(502),
            parent: Some(Slot(501)),
            root: None,
        });
        assert!(ev1.is_some());

        // Slot 501 arrives later out-of-order
        let ev2 = normalizer.normalize_slot(RawSlotNotification {
            slot: Slot(501),
            parent: Some(Slot(500)),
            root: None,
        });
        assert!(ev2.is_some());
        assert_eq!(ev2.unwrap().slot, Slot(501));

        // Both are stored without crashing
        assert_eq!(normalizer.seen_count(), 2);
    }
}
