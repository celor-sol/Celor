use chrono_bench::decision::{ExecutionDecisionEngine, RoutingAction};
use chrono_bench::freshness::FreshnessState;
use chrono_core::events::{ChronoEvent, ChronoEventKind};
use chrono_core::types::Slot;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SniperAction {
    Fire,
    Wait,
    Cancel,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SniperTriggerEvent {
    pub event_id: String,
    pub event_type: String,
    pub slot: Slot,
    pub signature: Option<String>,
    pub blockhash: Option<String>,
    pub bank_id: Option<String>,
    pub parent: Option<String>,
    pub leader: Option<String>,
    pub program_id: Option<String>,
    pub source: String,
    pub provenance: String,
    pub observed_at: u64,
    pub observed_at_nanos: Option<u64>,
    pub sequence: u64,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SniperRule {
    pub id: String,
    pub name: String,
    pub program: String,
    pub event_type: String,
    pub earliest_state: String,
    pub route: String,
    pub action: String,
    pub cancel_condition: String,
    pub leader_target: String,
    pub armed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SniperDecision {
    pub rule_id: String,
    pub event_id: String,
    pub action: SniperAction,
    pub reasons: Vec<String>,
    pub trigger_event: SniperTriggerEvent,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub execution_id: String,
    pub rule_id: String,
    pub status: String,
    pub transaction_base64: String,
    pub timestamp_ms: u64,
}

pub struct SniperEngine {
    rules: Mutex<HashMap<String, SniperRule>>,
    history: Mutex<VecDeque<SniperDecision>>,
    live_events: Mutex<VecDeque<SniperTriggerEvent>>,
    executions: Mutex<VecDeque<ExecutionRecord>>,
}

impl SniperEngine {
    pub fn new() -> Self {
        let mut rules = HashMap::new();
        
        rules.insert(
            "rule-default-catchall".to_string(),
            SniperRule {
                id: "rule-default-catchall".to_string(),
                name: "Global Catch-All Tracker".to_string(),
                program: "ANY".to_string(),
                event_type: "Leader Node Active".to_string(),
                earliest_state: "Candidate".to_string(),
                route: "QUIC".to_string(),
                action: "FIRE".to_string(),
                cancel_condition: "Bank Abandoned".to_string(),
                leader_target: "Current".to_string(),
                armed: true,
            },
        );

        Self {
            rules: Mutex::new(rules),
            history: Mutex::new(VecDeque::with_capacity(100)),
            live_events: Mutex::new(VecDeque::with_capacity(100)),
            executions: Mutex::new(VecDeque::with_capacity(100)),
        }
    }
}

impl Default for SniperEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SniperEngine {
    pub fn get_rules(&self) -> Vec<SniperRule> {
        let rules = self.rules.lock().unwrap();
        rules.values().cloned().collect()
    }

    pub fn add_rule(&self, rule: SniperRule) {
        let mut rules = self.rules.lock().unwrap();
        rules.insert(rule.id.clone(), rule);
    }

    pub fn update_rule(&self, id: &str, armed: bool) -> Option<SniperRule> {
        let mut rules = self.rules.lock().unwrap();
        if let Some(rule) = rules.get_mut(id) {
            rule.armed = armed;
            Some(rule.clone())
        } else {
            None
        }
    }

    pub fn delete_rule(&self, id: &str) -> bool {
        let mut rules = self.rules.lock().unwrap();
        rules.remove(id).is_some()
    }

    pub fn get_live_events(&self) -> Vec<SniperTriggerEvent> {
        let events = self.live_events.lock().unwrap();
        events.iter().cloned().collect()
    }

    pub fn get_recent_decisions(&self) -> Vec<SniperDecision> {
        let history = self.history.lock().unwrap();
        history.iter().cloned().collect()
    }

    pub fn record_execution(&self, execution_id: &str, rule_id: &str, status: &str, tx_base64: &str) {
        let record = ExecutionRecord {
            execution_id: execution_id.to_string(),
            rule_id: rule_id.to_string(),
            status: status.to_string(),
            transaction_base64: tx_base64.to_string(),
            timestamp_ms: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64,
        };
        let mut execs = self.executions.lock().unwrap();
        if execs.len() >= 100 {
            execs.pop_front();
        }
        execs.push_back(record);
    }

    pub fn update_execution_status(&self, execution_id: &str, status: &str) {
        let mut execs = self.executions.lock().unwrap();
        if let Some(record) = execs.iter_mut().find(|r| r.execution_id == execution_id) {
            record.status = status.to_string();
        }
    }

    pub fn get_executions(&self) -> Vec<ExecutionRecord> {
        let execs = self.executions.lock().unwrap();
        execs.iter().cloned().collect()
    }

    pub fn process_event(&self, event: &ChronoEvent, freshness: &FreshnessState) -> Option<SniperDecision> {
        let mut trigger = None;

        // Try to match specific event types for sniper triggers
        match &event.kind {
            ChronoEventKind::DeshredObserved { slot, signature, pre_execution_timestamp_nanos, .. } => {
                trigger = Some(SniperTriggerEvent {
                    event_id: format!("{}-{}", event.event_id, signature),
                    event_type: "Pre-Execution".to_string(),
                    slot: *slot,
                    signature: Some(signature.clone()),
                    blockhash: None,
                    bank_id: None,
                    parent: None,
                    leader: freshness.current_leader.clone(),
                    program_id: Some("Unknown".to_string()),
                    source: event.provider.to_string(),
                    provenance: "DIRECT".to_string(),
                    observed_at: event.received_time_ms,
                    observed_at_nanos: *pre_execution_timestamp_nanos,
                    sequence: event.event_id,
                    state: "PRE-EXECUTION".to_string(),
                });
            }
            ChronoEventKind::BankObserved { identity, parent, state: _ } => {
                trigger = Some(SniperTriggerEvent {
                    event_id: format!("{}-{}", event.event_id, identity.bank_id.as_ref().map(|b| b.0).unwrap_or(0)),
                    event_type: "Candidate Bank".to_string(),
                    slot: identity.slot,
                    signature: None,
                    blockhash: identity.blockhash.as_ref().map(|b| b.to_string()),
                    bank_id: identity.bank_id.as_ref().map(|b| b.to_string()),
                    parent: Some(match parent {
                        chrono_core::types::ParentReference::SlotOnly(s) => s.0.to_string(),
                        chrono_core::types::ParentReference::KnownBlockhash(b) => b.0.clone(),
                        chrono_core::types::ParentReference::Genesis => "Genesis".to_string(),
                        chrono_core::types::ParentReference::Unknown => "Unknown".to_string(),
                    }),
                    leader: freshness.current_leader.clone(),
                    program_id: None,
                    source: event.provider.to_string(),
                    provenance: "DIRECT".to_string(),
                    observed_at: event.received_time_ms,
                    observed_at_nanos: None,
                    sequence: event.event_id,
                    state: "CANDIDATE".to_string(),
                });
            }
            ChronoEventKind::UpdateParent { slot, cleared_bank_id, parent_slot, source, .. } => {
                trigger = Some(SniperTriggerEvent {
                    event_id: format!("{}-{}", event.event_id, slot.0),
                    event_type: "UpdateParent".to_string(),
                    slot: *slot,
                    signature: None,
                    blockhash: None,
                    bank_id: cleared_bank_id.as_ref().map(|b| b.to_string()),
                    parent: Some(parent_slot.0.to_string()),
                    leader: freshness.current_leader.clone(),
                    program_id: None,
                    source: source.clone(),
                    provenance: "DIRECT".to_string(),
                    observed_at: event.received_time_ms,
                    observed_at_nanos: None,
                    sequence: event.event_id,
                    state: "ABANDONED".to_string(),
                });
            }
            _ => {}
        }

        if let Some(t) = trigger {
            // Store in live events buffer
            {
                let mut evs = self.live_events.lock().unwrap();
                if evs.len() >= 100 {
                    evs.pop_front();
                }
                evs.push_back(t.clone());
            }

            // Evaluate against rules
            let rules = self.rules.lock().unwrap();
            for rule in rules.values() {
                if !rule.armed {
                    continue;
                }

                // Simple matching logic
                let mut matched = false;
                if rule.earliest_state == "Pre-Execution" && t.state == "PRE-EXECUTION" {
                    matched = true;
                }
                if rule.earliest_state == "Candidate" && (t.state == "CANDIDATE" || t.state == "PRE-EXECUTION") {
                    matched = true;
                }
                if t.state == "ABANDONED" {
                    matched = true;
                }

                if matched {
                    let mut reasons = Vec::new();
                    let mut action = SniperAction::Unknown;

                    if t.state == "ABANDONED" {
                        action = SniperAction::Cancel;
                        reasons.push("✕ Candidate bank abandoned".to_string());
                        reasons.push("✕ UpdateParent observed".to_string());
                        reasons.push("✕ Trigger no longer valid".to_string());
                    } else {
                        // Evaluate ExecutionDecisionEngine
                        let execution_decision = ExecutionDecisionEngine::evaluate_chrono(freshness, true);
                        match execution_decision.action {
                            RoutingAction::Submit => {
                                action = SniperAction::Fire;
                                reasons.push("✓ Target event matched".to_string());
                                reasons.push("✓ Candidate bank valid".to_string());
                                reasons.push("✓ Leader active".to_string());
                                reasons.push("✓ Fresh blockhash".to_string());
                                reasons.push("✓ QUIC route ready".to_string());
                            }
                            RoutingAction::Wait { wait_ms, .. } => {
                                action = SniperAction::Wait;
                                reasons.push("✓ Target event matched".to_string());
                                reasons.push(format!("⚠ Leader boundary near (wait {}ms)", wait_ms));
                                reasons.push("⚠ Next leader preparing".to_string());
                            }
                            RoutingAction::Abort { reason } => {
                                action = SniperAction::Cancel;
                                reasons.push(format!("✕ Execution engine aborted: {}", reason));
                            }
                            RoutingAction::Unknown { reason } => {
                                action = SniperAction::Unknown;
                                reasons.push(format!("✕ Insufficient evidence: {}", reason));
                            }
                            RoutingAction::Retry { .. } => {}
                        }
                    }

                    let decision = SniperDecision {
                        rule_id: rule.id.clone(),
                        event_id: t.event_id.clone(),
                        action,
                        reasons,
                        trigger_event: t.clone(),
                        timestamp_ms: t.observed_at,
                    };

                    let mut history = self.history.lock().unwrap();
                    if history.len() >= 100 {
                        history.pop_front();
                    }
                    history.push_back(decision.clone());

                    return Some(decision);
                }
            }
        }
        None
    }
}
