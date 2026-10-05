use crate::types::{BankId, Blockhash, ProviderId, Slot};
use serde::{Deserialize, Serialize};

/// Comprehensive Bank and Block Identity structure.
///
/// CRITICAL ARCHITECTURAL INVARIANT:
/// `bank_id` is STRICTLY local to `provider_id` (a single validator node instance).
/// Cross-provider reconciliation must NEVER equate two banks based on `bank_id` alone.
/// Cross-provider correlation requires `(slot, blockhash)` matching.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BankIdentity {
    pub provider_id: ProviderId,
    pub bank_id: Option<BankId>,
    pub slot: Slot,
    pub blockhash: Option<Blockhash>,
}

impl BankIdentity {
    pub fn new(
        provider_id: ProviderId,
        bank_id: Option<BankId>,
        slot: Slot,
        blockhash: Option<Blockhash>,
    ) -> Self {
        Self {
            provider_id,
            bank_id,
            slot,
            blockhash,
        }
    }

    /// Determines if two bank observations refer to the exact same physical block.
    ///
    /// Rules:
    /// 1. If from the SAME provider: matching `bank_id` means same bank.
    /// 2. If from DIFFERENT providers: `bank_id` is completely ignored. They are only the
    ///    same bank if both have identical `slot` and matching non-empty `blockhash`.
    pub fn correlates_with(&self, other: &Self) -> bool {
        if self.slot != other.slot {
            return false;
        }

        // Same provider path: local bank_id can be checked
        if self.provider_id == other.provider_id {
            if let (Some(a), Some(b)) = (&self.bank_id, &other.bank_id) {
                if a == b {
                    return true;
                }
            }
        }

        // Cross-provider reconciliation path: requires canonical blockhash
        match (&self.blockhash, &other.blockhash) {
            (Some(a), Some(b)) => a == b,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_bank_id_across_different_providers_not_same_bank() {
        let provider_a = ProviderId::new("provider-a-helius");
        let provider_b = ProviderId::new("provider-b-triton");

        // Both providers happen to emit local bank_id = 7 on slot 1000,
        // but they observed different candidate blocks (or hashes).
        let bank_a = BankIdentity::new(
            provider_a.clone(),
            Some(BankId(7)),
            Slot(1000),
            Some(Blockhash::new("BlockhashAlpha")),
        );

        let bank_b = BankIdentity::new(
            provider_b.clone(),
            Some(BankId(7)),
            Slot(1000),
            Some(Blockhash::new("BlockhashBeta")),
        );

        // Crucial invariant: bank_id 7 on Provider A != bank_id 7 on Provider B
        assert!(!bank_a.correlates_with(&bank_b));
        assert_ne!(bank_a, bank_b);
    }

    #[test]
    fn test_cross_provider_correlation_via_blockhash() {
        let provider_a = ProviderId::new("provider-a");
        let provider_b = ProviderId::new("provider-b");

        // Provider A assigned local bank_id = 12
        let bank_a = BankIdentity::new(
            provider_a,
            Some(BankId(12)),
            Slot(500),
            Some(Blockhash::new("CanonicalBlockhash42")),
        );

        // Provider B assigned local bank_id = 99
        let bank_b = BankIdentity::new(
            provider_b,
            Some(BankId(99)),
            Slot(500),
            Some(Blockhash::new("CanonicalBlockhash42")),
        );

        // They correlate globally because slot and blockhash match!
        assert!(bank_a.correlates_with(&bank_b));
    }

    #[test]
    fn test_different_slots_never_correlate() {
        let provider = ProviderId::new("provider-a");
        let bank_a = BankIdentity::new(
            provider.clone(),
            Some(BankId(1)),
            Slot(100),
            Some(Blockhash::new("Hash")),
        );
        let bank_b = BankIdentity::new(
            provider,
            Some(BankId(1)),
            Slot(101),
            Some(Blockhash::new("Hash")),
        );
        assert!(!bank_a.correlates_with(&bank_b));
    }
}
