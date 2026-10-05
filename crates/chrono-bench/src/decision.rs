use crate::freshness::{BankFreshnessTier, FreshnessState, FreshnessTier};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutingAction {
    Submit,
    Wait { wait_ms: u64, reason: String },
    Retry { attempt: u32, reason: String },
    Abort { reason: String },
    Unknown { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionResult {
    pub action: RoutingAction,
    pub explanation: String,
    pub evidence: Vec<String>,
}

pub struct ExecutionDecisionEngine;

impl ExecutionDecisionEngine {
    /// Evaluates decision for the CONTROL group (baseline conventional client).
    ///
    /// Conventional Solana clients lack leader-window awareness and candidate bank
    /// fork tracking. They unconditionally submit once signed.
    pub fn evaluate_control(freshness: &FreshnessState) -> DecisionResult {
        let is_expired = freshness.blockhash_tier == FreshnessTier::Stale;
        if is_expired {
            DecisionResult {
                action: RoutingAction::Abort {
                    reason: "Blockhash expired (>35s old)".to_string(),
                },
                explanation: "Control aborted: blockhash exceeded maximum validity window".to_string(),
                evidence: vec![format!("Blockhash age: {}ms", freshness.blockhash_age_ms)],
            }
        } else {
            DecisionResult {
                action: RoutingAction::Submit,
                explanation: "Control submitted: conventional client submits blindly without leader/bank timing awareness".to_string(),
                evidence: vec![
                    format!("Target blockhash: {} (age: {}ms)", freshness.blockhash, freshness.blockhash_age_ms),
                ],
            }
        }
    }

    /// Evaluates decision for the CHRONO-AWARE group.
    ///
    /// Exploits continuous slot clock, leader transition lookahead, and candidate
    /// bank fork invalidation to optimize submission reliability and avoid wasted fees.
    pub fn evaluate_chrono(freshness: &FreshnessState, leader_window_focus: bool) -> DecisionResult {
        // 1. Guard against abandoned candidate banks (Alpenglow UpdateParent state pruning)
        if freshness.bank_tier == BankFreshnessTier::Abandoned {
            return DecisionResult {
                action: RoutingAction::Abort {
                    reason: "Target parent candidate bank invalidated by UpdateParent".to_string(),
                },
                explanation: "Chrono aborted: candidate bank abandoned by leader handoff, preventing submission to dead fork".to_string(),
                evidence: vec![
                    format!("Candidate Bank ID: {:?}", freshness.bank_id),
                    "UpdateParent invalidated parent block".to_string(),
                ],
            };
        }

        // 2. Guard against stale blockhashes
        if freshness.blockhash_tier == FreshnessTier::Stale {
            return DecisionResult {
                action: RoutingAction::Abort {
                    reason: "Blockhash is stale/aging; requires stream refresh".to_string(),
                },
                explanation: "Chrono aborted: fresh blockhash required from stream before submission".to_string(),
                evidence: vec![format!("Blockhash age: {}ms", freshness.blockhash_age_ms)],
            };
        }

        // 3. Leader handoff optimization
        if leader_window_focus && freshness.leader_tier == FreshnessTier::HandoffImminent {
            let wait_ms = freshness.remaining_window_ms.max(15);
            return DecisionResult {
                action: RoutingAction::Wait {
                    wait_ms,
                    reason: format!(
                        "Leader handoff imminent (<40ms remaining in slot). Waiting for next leader: {:?}",
                        freshness.next_leader
                    ),
                },
                explanation: format!(
                    "Chrono waited {}ms: avoided late-slot submission likely to be dropped during leader handover",
                    wait_ms
                ),
                evidence: vec![
                    format!("Slot elapsed: {}ms / {}ms", freshness.slot_elapsed_ms, freshness.slot_target_duration_ms),
                    format!("Current leader: {:?}", freshness.current_leader),
                    format!("Next leader: {:?}", freshness.next_leader),
                ],
            };
        }

        // 4. Source staleness guard
        if freshness.source_tier == FreshnessTier::Stale {
            return DecisionResult {
                action: RoutingAction::Unknown {
                    reason: "Source telemetry stream delayed >2500ms; degraded certainty".to_string(),
                },
                explanation: "Chrono flagged degraded source: telemetry delayed beyond confidence threshold".to_string(),
                evidence: vec![format!("Last event received {}ms ago", freshness.last_event_received_ago_ms)],
            };
        }

        DecisionResult {
            action: RoutingAction::Submit,
            explanation: "Chrono submitted: verified fresh slot window, active leader tenure, and canonical parent lineage".to_string(),
            evidence: vec![
                format!("Slot: {}ms remaining", freshness.remaining_window_ms),
                format!("Leader: {:?}", freshness.current_leader),
                format!("Blockhash age: {}ms (FRESH)", freshness.blockhash_age_ms),
                format!("Bank state: {:?}", freshness.bank_tier),
            ],
        }
    }
}
