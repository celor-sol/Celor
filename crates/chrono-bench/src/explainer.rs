use crate::experiment::SampleRecord;
use crate::storage::BenchmarkStorage;

pub struct ExecutionExplainer;

impl ExecutionExplainer {
    /// Finds a sample by execution ID across all stored experiments.
    pub fn find_sample(execution_id: &str) -> Option<SampleRecord> {
        let storage = BenchmarkStorage::default();
        let exp_ids = storage.list_experiments();

        for exp_id in exp_ids.iter().rev() {
            if let Some(samples) = storage.load_samples(exp_id) {
                if let Some(sample) = samples.into_iter().find(|s| s.execution_id == execution_id) {
                    return Some(sample);
                }
            }
        }
        None
    }

    /// Reconstructs the complete decision and execution timeline for a given execution ID.
    pub fn find_and_explain(execution_id: &str) -> Option<String> {
        Self::find_sample(execution_id).map(|s| Self::format_explanation(&s))
    }

    pub fn format_explanation(sample: &SampleRecord) -> String {
        format!(
            r#"============================================================
CHRONO EXECUTION EXPLAINER — {exec_id}
============================================================
Experiment ID:     {exp_id}
Group:             {group}
Sample Index:      #{idx}
Provenance:        [{prov}]

1. STATE AT DECISION TIME (T0)
   Slot:           {slot}
   Remaining Time: {rem_ms}ms
   Current Leader: {leader}
   Next Leader:    {next_leader}
   Blockhash Age:  {bh_age}ms
   Candidate Bank: {bank}
   UpdateParent:   {up}

2. ROUTING DECISION & ACTION
   Action:         {status:?}
   Reason:         {reason}
   Comparative:    {comp_exp}

3. TIMELINE & LATENCIES
   Build -> Sign:  {build_sign:.2}µs
   Sign -> Submit: {sign_sub:.2}µs
   Submit -> Ack:  {sub_ack}
   Submit -> Land: {sub_land}
   Submit -> Conf: {sub_conf}
   Landed Slot:    {land_slot} (Delta: {slot_delta} slots)

4. CONFIRMATION STATUS
   Status:         {status:?}
   Signature:      {sig}
   Error (if any): {err}
============================================================"#,
            exec_id = sample.execution_id,
            exp_id = sample.experiment_id,
            group = sample.group,
            idx = sample.sample_index,
            prov = sample.provenance,
            slot = sample.slot_at_submission,
            rem_ms = sample.remaining_slot_time_ms.unwrap_or(0),
            leader = sample.leader_at_submission.as_deref().unwrap_or("UNAVAILABLE"),
            next_leader = if sample.landed_on_intended_leader.unwrap_or(false) {
                "Intended Leader"
            } else {
                "Handoff Boundary"
            },
            bh_age = sample.blockhash_age_ms,
            bank = sample.candidate_bank_id.as_deref().unwrap_or("Public RPC (single block)"),
            up = if sample.update_parent_observed { "OBSERVED (state pruned)" } else { "None" },
            reason = sample.decision_reason,
            comp_exp = sample.comparative_explanation,
            build_sign = sample.timestamps.build_to_sign_ns as f64 / 1_000.0,
            sign_sub = sample.timestamps.sign_to_submit_ns as f64 / 1_000.0,
            sub_ack = sample.timestamps.submit_to_ack_ns.map(|ns| format!("{:.2}ms", ns as f64 / 1_000_000.0)).unwrap_or_else(|| "N/A".into()),
            sub_land = sample.timestamps.submit_to_landed_ms.map(|ms| format!("{:.1}ms", ms)).unwrap_or_else(|| "N/A".into()),
            sub_conf = sample.timestamps.submit_to_confirmed_ms.map(|ms| format!("{:.1}ms", ms)).unwrap_or_else(|| "N/A".into()),
            land_slot = sample.slot_at_landing.map(|s| s.to_string()).unwrap_or_else(|| "N/A".into()),
            slot_delta = sample.slot_delta_landed.map(|d| d.to_string()).unwrap_or_else(|| "N/A".into()),
            status = sample.status,
            sig = sample.signature.as_deref().unwrap_or("None"),
            err = sample.error_reason.as_deref().unwrap_or("None")
        )
    }
}
