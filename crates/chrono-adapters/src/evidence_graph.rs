use chrono_core::types::{Blockhash, FieldProvenance, ProviderConflictStatus, ProviderId, Slot};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An individual field observation recorded from an external source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceObservation {
    pub source_id: ProviderId,
    pub observed_at_nanos: u64,
    pub field_name: String,
    pub value: String,
    pub provenance: FieldProvenance,
}

/// Consolidated evidence across multiple providers for a given (slot, blockhash).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotEvidence {
    pub slot: Slot,
    pub blockhash: Option<Blockhash>,
    pub observations: Vec<SourceObservation>,
    pub first_observed_by: Option<ProviderId>,
    pub first_observed_nanos: Option<u64>,
    pub authoritative_bank_id: Option<u64>,
    pub authoritative_bank_hash: Option<String>,
    pub authoritative_status: String,
    pub has_conflicts: bool,
    pub conflict_summary: Option<String>,
    /// Classification standard: MATCH, CONFLICT, UNRESOLVED, RESOLVED (Section 22)
    pub conflict_status: ProviderConflictStatus,
    pub authoritative_source: Option<ProviderId>,
    pub resolution_reason: Option<String>,
}

/// Source Evidence Graph for cross-source reconciliation and conflict resolution.
///
/// Invariant: NEVER overwrite conflicting evidence.
/// Preserve: "Source A observed X at T1, Source B observed Y at T2"
/// and apply field-specific authority rules to determine canonical state.
#[derive(Debug, Default)]
pub struct SourceEvidenceGraph {
    evidence_by_slot: HashMap<u64, SlotEvidence>,
}

impl SourceEvidenceGraph {
    pub fn new() -> Self {
        Self {
            evidence_by_slot: HashMap::new(),
        }
    }

    /// Records an observation from a specific provider with strict deduplication and conflict tracking.
    #[allow(clippy::too_many_arguments)]
    pub fn record_observation(
        &mut self,
        source: ProviderId,
        slot: Slot,
        blockhash: Option<Blockhash>,
        field_name: &str,
        value: &str,
        provenance: FieldProvenance,
        timestamp_nanos: u64,
    ) {
        let entry = self.evidence_by_slot.entry(slot.as_u64()).or_insert_with(|| SlotEvidence {
            slot,
            blockhash: blockhash.clone(),
            observations: Vec::new(),
            first_observed_by: Some(source.clone()),
            first_observed_nanos: Some(timestamp_nanos),
            authoritative_bank_id: None,
            authoritative_bank_hash: None,
            authoritative_status: "Observed".to_string(),
            has_conflicts: false,
            conflict_summary: None,
            conflict_status: ProviderConflictStatus::Match,
            authoritative_source: Some(source.clone()),
            resolution_reason: None,
        });

        // 1. Deduplication (Section 21): Same event from same provider is never double-counted
        for existing in &mut entry.observations {
            if existing.source_id == source && existing.field_name == field_name && existing.value == value {
                // Update observed timestamp if earlier
                if timestamp_nanos < existing.observed_at_nanos {
                    existing.observed_at_nanos = timestamp_nanos;
                }
                return;
            }
        }

        if entry.blockhash.is_none() && blockhash.is_some() {
            entry.blockhash = blockhash;
        }

        if let Some(first_ts) = entry.first_observed_nanos {
            if timestamp_nanos < first_ts {
                entry.first_observed_nanos = Some(timestamp_nanos);
                entry.first_observed_by = Some(source.clone());
            }
        }

        let obs = SourceObservation {
            source_id: source.clone(),
            observed_at_nanos: timestamp_nanos,
            field_name: field_name.to_string(),
            value: value.to_string(),
            provenance,
        };

        // 2. Conflict Detection across providers (Section 20 & 22)
        let mut detected_conflict = false;
        for existing in &entry.observations {
            if existing.field_name == field_name && existing.value != value && existing.source_id != source {
                detected_conflict = true;
                entry.has_conflicts = true;
                entry.conflict_summary = Some(format!(
                    "Conflict on field '{}': {} reported '{}' while {} reported '{}'",
                    field_name, existing.source_id, existing.value, source, value
                ));
            }
        }

        // 3. Protocol-governed authoritative resolution
        if field_name == "bank_id" {
            if let Ok(bid) = value.parse::<u64>() {
                if source.as_str().contains("geyser") || entry.authoritative_bank_id.is_none() {
                    entry.authoritative_bank_id = Some(bid);
                    entry.authoritative_source = Some(source.clone());
                }
            }
        } else if field_name == "bank_hash" {
            if source.as_str().contains("geyser") || entry.authoritative_bank_hash.is_none() {
                entry.authoritative_bank_hash = Some(value.to_string());
                entry.authoritative_source = Some(source.clone());
            }
        } else if field_name == "status" {
            if value == "Rooted" {
                entry.authoritative_status = "Rooted".to_string();
                entry.authoritative_source = Some(source.clone());
                entry.resolution_reason = Some("Promoted to Rooted by authoritative provider".to_string());
                entry.conflict_status = ProviderConflictStatus::Resolved;
            } else if value == "Confirmed" && entry.authoritative_status != "Rooted" {
                entry.authoritative_status = "Confirmed".to_string();
                entry.authoritative_source = Some(source.clone());
                entry.resolution_reason = Some("Promoted to Confirmed".to_string());
                if entry.has_conflicts {
                    entry.conflict_status = ProviderConflictStatus::Resolved;
                }
            }
        } else if field_name == "blockhash" && detected_conflict {
            if entry.authoritative_status == "Rooted" || entry.authoritative_status == "Confirmed" {
                entry.conflict_status = ProviderConflictStatus::Resolved;
                entry.resolution_reason = Some("Resolved via canonical block confirmation".to_string());
            } else {
                entry.conflict_status = ProviderConflictStatus::Conflict;
            }
        }

        if detected_conflict && entry.conflict_status != ProviderConflictStatus::Resolved {
            entry.conflict_status = ProviderConflictStatus::Conflict;
        }

        entry.observations.push(obs);
    }

    pub fn get_slot_evidence(&self, slot: Slot) -> Option<&SlotEvidence> {
        self.evidence_by_slot.get(&slot.as_u64())
    }

    pub fn all_evidence(&self) -> Vec<&SlotEvidence> {
        self.evidence_by_slot.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_source_conflict_preserved_and_reconciled() {
        let mut graph = SourceEvidenceGraph::new();
        let slot = Slot(500);

        // RPC reports confirmed blockhash A
        graph.record_observation(
            ProviderId::new("testnet-rpc"),
            slot,
            Some(Blockhash::new("Blockhash_A")),
            "blockhash",
            "Blockhash_A",
            FieldProvenance::Direct,
            1000,
        );

        // Validator Geyser reports bank_id = 42 and bank_hash = Hash_B
        graph.record_observation(
            ProviderId::new("local-validator"),
            slot,
            Some(Blockhash::new("Blockhash_A")),
            "bank_id",
            "42",
            FieldProvenance::Direct,
            900, // Geyser saw it 100ns faster
        );

        let evidence = graph.get_slot_evidence(slot).expect("Evidence should exist");
        assert_eq!(evidence.first_observed_by, Some(ProviderId::new("local-validator")));
        assert_eq!(evidence.authoritative_bank_id, Some(42));
        assert_eq!(evidence.observations.len(), 2);
        assert!(!evidence.has_conflicts);

        // Conflicting report: another provider says bank_id = 99
        graph.record_observation(
            ProviderId::new("yellowstone-node2"),
            slot,
            Some(Blockhash::new("Blockhash_A")),
            "bank_id",
            "99",
            FieldProvenance::Direct,
            1100,
        );

        let updated_evidence = graph.get_slot_evidence(slot).expect("Evidence should exist");
        assert!(updated_evidence.has_conflicts);
        assert!(updated_evidence.conflict_summary.as_ref().unwrap().contains("Conflict on field 'bank_id'"));
        // Geyser still held as authoritative
        assert_eq!(updated_evidence.authoritative_bank_id, Some(42));
    }

    #[test]
    fn test_multi_provider_blockhash_conflict_resolution_and_deduplication() {
        let mut graph = SourceEvidenceGraph::new();
        let slot = Slot(700);

        // Provider A: slot 700 / Block A
        graph.record_observation(
            ProviderId::new("provider-a"),
            slot,
            Some(Blockhash::new("Block_A")),
            "blockhash",
            "Block_A",
            FieldProvenance::Direct,
            1000,
        );

        // Deduplication test: Provider A sends the exact same observation again
        graph.record_observation(
            ProviderId::new("provider-a"),
            slot,
            Some(Blockhash::new("Block_A")),
            "blockhash",
            "Block_A",
            FieldProvenance::Direct,
            1050,
        );
        let ev = graph.get_slot_evidence(slot).unwrap();
        assert_eq!(ev.observations.len(), 1, "Duplicate observation must not be double counted");
        assert_eq!(ev.conflict_status, ProviderConflictStatus::Match);

        // Provider B: slot 700 / Block A (agreement)
        graph.record_observation(
            ProviderId::new("provider-b"),
            slot,
            Some(Blockhash::new("Block_A")),
            "blockhash",
            "Block_A",
            FieldProvenance::Direct,
            1010,
        );
        let ev = graph.get_slot_evidence(slot).unwrap();
        assert_eq!(ev.observations.len(), 2);
        assert_eq!(ev.conflict_status, ProviderConflictStatus::Match);

        // Provider C: slot 700 / Block B (divergence / candidate fork)
        graph.record_observation(
            ProviderId::new("provider-c"),
            slot,
            Some(Blockhash::new("Block_B")),
            "blockhash",
            "Block_B",
            FieldProvenance::Direct,
            1020,
        );
        let ev = graph.get_slot_evidence(slot).unwrap();
        assert_eq!(ev.observations.len(), 3);
        assert!(ev.has_conflicts);
        assert_eq!(ev.conflict_status, ProviderConflictStatus::Conflict);

        // Authoritative confirmation arrives promoting Block_A to Rooted
        graph.record_observation(
            ProviderId::new("local-geyser"),
            slot,
            Some(Blockhash::new("Block_A")),
            "status",
            "Rooted",
            FieldProvenance::Direct,
            1030,
        );
        let ev = graph.get_slot_evidence(slot).unwrap();
        assert_eq!(ev.conflict_status, ProviderConflictStatus::Resolved);
        assert_eq!(ev.authoritative_status, "Rooted");
        assert!(ev.resolution_reason.as_ref().unwrap().contains("authoritative"));
    }
}
