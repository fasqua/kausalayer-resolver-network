use anchor_lang::prelude::*;

/// Bet commitment account for each bettor per market.
/// PDA seeds: [b"commitment", market_id, bettor_pubkey]
#[account]
pub struct BetCommitment {
    /// Market being bet on
    pub market_id: [u8; 32],
    /// Poseidon hash of the commitment
    /// commitment = Poseidon(market_id, outcome, amount, secret_nonce, owner_pubkey)
    pub commitment_hash: [u8; 32],
    /// Chosen side: 0=NO, 1=YES (public for pool tracking)
    pub side: u8,
    /// Bet amount in lamports (public for pool tracking)
    pub amount: u64,
    /// Whether winnings have been claimed
    pub claimed: bool,
    /// PDA bump seed
    pub bump: u8,
}

impl BetCommitment {
    /// 8 (discriminator) + 32 + 32 + 1 + 8 + 1 + 1 = 83
    pub const SIZE: usize = 8 + 32 + 32 + 1 + 8 + 1 + 1;
}

/// Nullifier account to prevent double-claiming.
/// PDA seeds: [b"nullifier", market_id, nullifier_hash]
#[account]
pub struct NullifierAccount {
    /// Associated market
    pub market_id: [u8; 32],
    /// Nullifier hash
    pub nullifier: [u8; 32],
    /// Unix timestamp when claimed
    pub claimed_at: i64,
    /// PDA bump seed
    pub bump: u8,
}

impl NullifierAccount {
    /// 8 (discriminator) + 32 + 32 + 8 + 1 = 81
    pub const SIZE: usize = 8 + 32 + 32 + 8 + 1;
}
