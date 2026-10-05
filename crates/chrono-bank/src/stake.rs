use chrono_core::types::Slot;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

/// Calculation integrity status for certificate stake verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StakeCalculationStatus {
    /// Exact participation calculated from verified active validator set.
    ExactVerified,
    /// Some participant ranks in bitmap were not found in active set.
    PartialSignersUnknown,
    /// Active validator set for the requested epoch is missing.
    ValidatorSetMissing,
    /// Slot context does not match recorded epoch boundaries.
    EpochMismatch,
}

/// Individual validator record within an epoch's active stake set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatorStakeEntry {
    pub identity: String,
    pub stake_lamports: u64,
    pub is_active: bool,
    pub rank: usize,
    pub bls_pubkey: Option<Vec<u8>>,
}

/// Complete active stake table for a single epoch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpochStakeTable {
    pub epoch: u64,
    pub total_active_stake_lamports: u64,
    pub total_validators: usize,
    pub active_validators: usize,
    pub validators_by_rank: HashMap<usize, ValidatorStakeEntry>,
    pub validators_by_identity: HashMap<String, usize>,
    pub created_at_nanos: u64,
}

impl EpochStakeTable {
    pub fn new(epoch: u64, mut validators: Vec<ValidatorStakeEntry>, created_at_nanos: u64) -> Self {
        let mut total_active: u64 = 0;
        let mut active_count = 0;
        let mut by_rank = HashMap::with_capacity(validators.len());
        let mut by_id = HashMap::with_capacity(validators.len());

        for (idx, mut val) in validators.drain(..).enumerate() {
            val.rank = idx;
            if val.is_active {
                total_active = total_active.saturating_add(val.stake_lamports);
                active_count += 1;
            }
            by_id.insert(val.identity.clone(), idx);
            by_rank.insert(idx, val);
        }

        Self {
            epoch,
            total_active_stake_lamports: total_active,
            total_validators: by_rank.len(),
            active_validators: active_count,
            validators_by_rank: by_rank,
            validators_by_identity: by_id,
            created_at_nanos,
        }
    }
}

/// Point-in-time validator stake audit snapshot (Section 9).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StakeSnapshot {
    pub epoch: u64,
    pub slot_context: Slot,
    pub validator_identity: String,
    pub stake_lamports: u64,
    pub active_status: bool,
    pub source: String,
    pub snapshot_time_nanos: u64,
}

/// Formal mathematical participation evaluation result.
/// Invariant: All calculations use integer-safe arithmetic; ZERO floats used for acceptance decisions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StakeParticipationResult {
    pub epoch: u64,
    pub slot_context: Slot,
    pub total_active_stake_lamports: u64,
    pub participating_stake_lamports: u64,
    /// Integer-safe basis points (10,000 = 100.00%).
    pub participation_basis_points: u32,
    /// Required threshold in basis points (e.g. 8000 for Fast Finalize, 6000 for Fallback).
    pub required_threshold_basis_points: u32,
    /// Final consensus condition: participation_basis_points >= required_threshold_basis_points.
    pub threshold_met: bool,
    pub unknown_stake_lamports: u64,
    pub calculation_status: StakeCalculationStatus,
}

/// Epoch-aware Stake Engine managing validator sets and participation math.
#[derive(Debug, Default, Clone)]
pub struct StakeEngine {
    tables_by_epoch: BTreeMap<u64, EpochStakeTable>,
}

impl StakeEngine {
    pub fn new() -> Self {
        Self {
            tables_by_epoch: BTreeMap::new(),
        }
    }

    /// Records the active validator stake table for an epoch.
    pub fn register_epoch_stake(&mut self, table: EpochStakeTable) {
        self.tables_by_epoch.insert(table.epoch, table);
    }

    /// Alias for register_epoch_stake.
    pub fn insert_epoch_table(&mut self, table: EpochStakeTable) {
        self.register_epoch_stake(table);
    }

    /// Retrieves the exact stake table for an epoch.
    /// Invariant: Never silently substitute the newest stake table when an older epoch is queried.
    pub fn get_epoch_table(&self, epoch: u64) -> Option<&EpochStakeTable> {
        self.tables_by_epoch.get(&epoch)
    }

    /// Evaluates certificate participation across decoded validator signer ranks.
    pub fn evaluate_participation(
        &self,
        epoch: u64,
        slot_context: Slot,
        signer_ranks: &[usize],
        required_threshold_bps: u32,
    ) -> StakeParticipationResult {
        let table = match self.get_epoch_table(epoch) {
            Some(t) => t,
            None => {
                return StakeParticipationResult {
                    epoch,
                    slot_context,
                    total_active_stake_lamports: 0,
                    participating_stake_lamports: 0,
                    participation_basis_points: 0,
                    required_threshold_basis_points: required_threshold_bps,
                    threshold_met: false,
                    unknown_stake_lamports: 0,
                    calculation_status: StakeCalculationStatus::ValidatorSetMissing,
                };
            }
        };

        if table.total_active_stake_lamports == 0 {
            return StakeParticipationResult {
                epoch,
                slot_context,
                total_active_stake_lamports: 0,
                participating_stake_lamports: 0,
                participation_basis_points: 0,
                required_threshold_basis_points: required_threshold_bps,
                threshold_met: false,
                unknown_stake_lamports: 0,
                calculation_status: StakeCalculationStatus::ValidatorSetMissing,
            };
        }

        let mut participating_stake: u64 = 0;
        let mut unknown_count: usize = 0;

        for rank in signer_ranks {
            if let Some(entry) = table.validators_by_rank.get(rank) {
                if entry.is_active {
                    participating_stake = participating_stake.saturating_add(entry.stake_lamports);
                }
            } else {
                unknown_count += 1;
            }
        }

        // Integer-safe basis points calculation: (participating * 10,000) / total_active
        // Uses u128 intermediate to prevent 64-bit overflow during multiplication
        let part_u128 = participating_stake as u128;
        let total_u128 = table.total_active_stake_lamports as u128;
        let bps = ((part_u128.saturating_mul(10_000)) / total_u128) as u32;

        let threshold_met = bps >= required_threshold_bps;
        let calculation_status = if unknown_count > 0 {
            StakeCalculationStatus::PartialSignersUnknown
        } else {
            StakeCalculationStatus::ExactVerified
        };

        StakeParticipationResult {
            epoch,
            slot_context,
            total_active_stake_lamports: table.total_active_stake_lamports,
            participating_stake_lamports: participating_stake,
            participation_basis_points: bps,
            required_threshold_basis_points: required_threshold_bps,
            threshold_met,
            unknown_stake_lamports: (unknown_count as u64).saturating_mul(1_000_000_000), // nominal tracking
            calculation_status,
        }
    }

    /// Produces point-in-time StakeSnapshot for forensic logging.
    pub fn build_snapshot(
        &self,
        epoch: u64,
        slot_context: Slot,
        validator_identity: &str,
        source: &str,
        snapshot_time_nanos: u64,
    ) -> Option<StakeSnapshot> {
        let table = self.get_epoch_table(epoch)?;
        let rank = table.validators_by_identity.get(validator_identity)?;
        let entry = table.validators_by_rank.get(rank)?;

        Some(StakeSnapshot {
            epoch,
            slot_context,
            validator_identity: entry.identity.clone(),
            stake_lamports: entry.stake_lamports,
            active_status: entry.is_active,
            source: source.to_string(),
            snapshot_time_nanos,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_mock_validators(count: usize, stake_per_val: u64) -> Vec<ValidatorStakeEntry> {
        (0..count)
            .map(|i| ValidatorStakeEntry {
                identity: format!("val-{}", i),
                stake_lamports: stake_per_val,
                is_active: true,
                rank: i,
                bls_pubkey: None,
            })
            .collect()
    }

    #[test]
    fn test_exact_integer_participation_calculation() {
        let mut engine = StakeEngine::new();
        // 10 validators each with 100 SOL (100_000_000_000 lamports) -> Total = 1,000 SOL
        let validators = create_mock_validators(10, 100_000_000_000);
        let table = EpochStakeTable::new(500, validators, 1000);
        engine.register_epoch_stake(table);

        // Signers: 8 validators (80% stake)
        let signers = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let result = engine.evaluate_participation(500, Slot(100), &signers, 8000);

        assert_eq!(result.total_active_stake_lamports, 1_000_000_000_000);
        assert_eq!(result.participating_stake_lamports, 800_000_000_000);
        assert_eq!(result.participation_basis_points, 8000);
        assert!(result.threshold_met);
        assert_eq!(result.calculation_status, StakeCalculationStatus::ExactVerified);
    }

    #[test]
    fn test_epoch_transition_correctness_never_mixes_tables() {
        let mut engine = StakeEngine::new();
        // Epoch 500: 10 validators @ 100 SOL each = 1,000 SOL
        let v500 = create_mock_validators(10, 100_000_000_000);
        engine.register_epoch_stake(EpochStakeTable::new(500, v500, 1000));

        // Epoch 501: Stake changes drastically! 10 validators @ 200 SOL each = 2,000 SOL
        // and Validator 0 deactivates!
        let mut v501 = create_mock_validators(10, 200_000_000_000);
        v501[0].is_active = false;
        engine.register_epoch_stake(EpochStakeTable::new(501, v501, 2000));

        // Same signers [0, 1, 2, 3, 4] (5 validators)
        // Under Epoch 500: 5/10 = 50.00% (5000 bps)
        let r500 = engine.evaluate_participation(500, Slot(100), &[0, 1, 2, 3, 4], 6000);
        assert_eq!(r500.participation_basis_points, 5000);
        assert!(!r500.threshold_met);

        // Under Epoch 501: Val 0 is inactive! Only vals 1..4 count (800 SOL / 1800 SOL active)
        // 800 * 10,000 / 1800 = 4444 bps (44.44%)
        let r501 = engine.evaluate_participation(501, Slot(100), &[0, 1, 2, 3, 4], 6000);
        assert_eq!(r501.participation_basis_points, 4444);
        assert!(!r501.threshold_met);

        // Prove querying missing epoch returns ValidatorSetMissing
        let r502 = engine.evaluate_participation(502, Slot(100), &[0, 1, 2], 6000);
        assert_eq!(r502.calculation_status, StakeCalculationStatus::ValidatorSetMissing);
        assert!(!r502.threshold_met);
    }
}
