use chrono_core::types::{LeaderId, Slot};
use std::collections::BTreeMap;

/// Engine maintaining cached and active slot leader assignments.
///
/// CRITICAL INVARIANTS:
/// 1. Continuously warm schedule: `leader_for(slot)` must never perform network I/O.
/// 2. Deterministic transitions: Detects leader changes across consecutive slots.
#[derive(Debug, Default, Clone)]
pub struct LeaderEngine {
    /// In-memory cache mapping slot -> Leader identity
    schedule: BTreeMap<Slot, LeaderId>,
    /// Currently active leader
    current_leader: Option<(Slot, LeaderId)>,
}

impl LeaderEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets or updates the leader for a specific slot.
    pub fn set_leader(&mut self, slot: Slot, leader: LeaderId) {
        self.schedule.insert(slot, leader.clone());
        self.current_leader = Some((slot, leader));
    }

    /// Pre-populates the cache with a block of leader assignments (e.g. from getLeaderSchedule).
    pub fn populate_schedule(&mut self, starting_slot: Slot, leaders: Vec<LeaderId>) {
        for (idx, leader) in leaders.into_iter().enumerate() {
            let slot = Slot(starting_slot.as_u64().saturating_add(idx as u64));
            self.schedule.insert(slot, leader);
        }
    }

    /// Queries the leader for any slot from the warm cache WITHOUT network I/O.
    pub fn leader_for(&self, slot: Slot) -> Option<LeaderId> {
        self.schedule.get(&slot).cloned()
    }

    /// Returns the currently active leader if known.
    pub fn current_leader(&self) -> Option<LeaderId> {
        self.current_leader.as_ref().map(|(_, l)| l.clone())
    }

    /// Returns the next scheduled leader following `current_slot`.
    pub fn next_leader(&self, current_slot: Slot) -> Option<LeaderId> {
        self.schedule.get(&current_slot.next()).cloned()
    }

    /// Returns true if consecutive slots have different leaders.
    pub fn is_leader_transition(&self, slot_a: Slot, slot_b: Slot) -> bool {
        match (self.leader_for(slot_a), self.leader_for(slot_b)) {
            (Some(a), Some(b)) => a != b,
            _ => false,
        }
    }

    /// Cleans up schedule entries older than `min_slot`.
    pub fn prune_older_than(&mut self, min_slot: Slot) {
        self.schedule.retain(|s, _| *s >= min_slot);
    }

    /// Returns count of scheduled slots cached in memory.
    pub fn cached_slots_count(&self) -> usize {
        self.schedule.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leader_engine_warm_lookup_and_transition() {
        let mut engine = LeaderEngine::new();
        let slot = Slot(100);

        let leader1 = LeaderId::new("LeaderAlpha");
        let leader2 = LeaderId::new("LeaderBeta");

        engine.set_leader(slot, leader1.clone());
        engine.set_leader(slot.next(), leader2.clone());

        assert_eq!(engine.leader_for(slot), Some(leader1.clone()));
        assert_eq!(engine.next_leader(slot), Some(leader2.clone()));
        assert!(engine.is_leader_transition(slot, slot.next()));

        // Pruning works
        engine.prune_older_than(Slot(101));
        assert!(engine.leader_for(slot).is_none());
        assert_eq!(engine.leader_for(slot.next()), Some(leader2));
    }
}
