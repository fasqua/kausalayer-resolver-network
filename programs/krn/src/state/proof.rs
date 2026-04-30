use anchor_lang::prelude::*;

/// Account for each zkTLS proof submission.
/// PDA seeds: [b"proof", market_id, &[source_index]]
#[account]
pub struct ProofSubmission {
    /// Market being resolved
    pub market_id: [u8; 32],
    /// Index of the source config being proven
    pub source_index: u8,
    /// Claimed outcome: 0=NO, 1=YES
    pub claimed_outcome: u8,
    /// Public key of the submitter
    pub submitter: Pubkey,
    /// Unix timestamp of submission
    pub timestamp: i64,
    /// Hash of the proof data
    pub proof_hash: [u8; 32],
    /// Whether the proof has been verified as valid
    pub verified: bool,
    /// PDA bump seed
    pub bump: u8,
}

impl ProofSubmission {
    /// 8 (discriminator) + 32 + 1 + 1 + 32 + 8 + 32 + 1 + 1 = 116
    pub const SIZE: usize = 8 + 32 + 1 + 1 + 32 + 8 + 32 + 1 + 1;
}
