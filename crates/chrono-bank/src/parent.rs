use crate::graph::{BankGraph, BankKey};
use chrono_core::types::{Blockhash, ParentReference, Slot};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParentChangeNotification {
    pub bank_key: BankKey,
    pub slot: Slot,
    pub previous_parent: Option<ParentReference>,
    pub new_parent: ParentReference,
}

/// Engine tracking and validating parent lineage across slots and candidate banks.
#[derive(Debug, Default)]
pub struct ParentEngine;

impl ParentEngine {
    pub fn new() -> Self {
        Self
    }

    /// Updates or registers a parent reference for an existing bank node.
    /// If the parent differs from a previously recorded parent, returns a `ParentChangeNotification`.
    pub fn update_parent(
        &self,
        graph: &mut BankGraph,
        bank_key: &BankKey,
        new_parent: ParentReference,
    ) -> Option<ParentChangeNotification> {
        let node = graph.get_bank_mut(bank_key)?;

        let previous_parent = node.parent.clone();

        match &previous_parent {
            Some(prev) if prev == &new_parent => {
                // No change
                None
            }
            _ => {
                node.parent = Some(new_parent.clone());
                Some(ParentChangeNotification {
                    bank_key: bank_key.clone(),
                    slot: node.identity.slot,
                    previous_parent,
                    new_parent,
                })
            }
        }
    }

    /// Resolves the parent blockhash for a bank if known.
    pub fn resolve_parent_blockhash(
        &self,
        graph: &BankGraph,
        bank_key: &BankKey,
    ) -> Option<Blockhash> {
        let node = graph.get_bank(bank_key)?;
        match &node.parent {
            Some(ParentReference::KnownBlockhash(hash)) => Some(hash.clone()),
            _ => None,
        }
    }
}
