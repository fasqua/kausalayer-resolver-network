use anchor_lang::prelude::*;

/// Data source configuration for zkTLS verification.
/// Each market requires at least 3 source configs.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct SourceConfig {
    /// Hash of the target domain (e.g., hash("api.coinbase.com"))
    pub domain_hash: [u8; 32],
    /// Hash of the valid API path for this market
    pub path_hash: [u8; 32],
    /// Hash of the JSONPath used to extract outcome value
    pub json_path_hash: [u8; 32],
}

/// Market lifecycle state.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum MarketState {
    /// Betting is open
    Open,
    /// Betting closed, awaiting resolution
    Closed,
    /// Collecting proofs
    Resolving,
    /// Finalized, winners can claim
    Resolved,
    /// Resolution failed, entering fallback
    Failed,
}

/// Primary account for each market resolved by KRN.
/// PDA seeds: [b"market", market_id]
#[account]
pub struct MarketAccount {
    /// Unique market identifier hash
    pub market_id: [u8; 32],
    /// Platform that created the market
    pub creator: Pubkey,
    /// Unix timestamp when betting closes
    pub close_timestamp: i64,
    /// Unix timestamp deadline for resolution
    pub resolution_deadline: i64,
    /// Current market lifecycle state
    pub state: MarketState,
    /// Resolved outcome: 0=NO, 1=YES, 255=unresolved
    pub outcome: u8,
    /// Number of valid proofs submitted so far
    pub source_count: u8,
    /// Minimum sources required to resolve (default 3)
    pub required_sources: u8,
    /// Number of active source configs
    pub num_sources: u8,
    /// Source configurations (max 10)
    pub source_configs: [SourceConfig; 10],
    /// Merkle root of all bet commitments (computed on-chain)
    pub commitment_root: [u8; 32],
    /// Number of commitments inserted into the Merkle tree
    pub commitment_count: u32,
    /// Incremental Merkle tree: hash at each level (depth 10, max 1024 bets)
    pub commitment_tree: [[u8; 32]; 10],
    /// Total funds in market pool (lamports)
    pub total_pool: u64,
    /// Funds in YES pool (lamports)
    pub yes_pool: u64,
    /// Funds in NO pool (lamports)
    pub no_pool: u64,
    /// Number of nullifiers already used
    pub nullifier_count: u16,
    /// Authority for fallback resolution
    pub authority: Pubkey,
    /// Price threshold for market resolution (e.g., 100000 for BTC > 100k)
    pub threshold: u64,
    /// Comparison type: 0=greater_than, 1=less_than, 2=equal
    pub comparison: u8,
    /// PDA bump seed
    pub bump: u8,
}

impl MarketAccount {
    /// Account size in bytes.
    /// 8 (discriminator) + 32 + 32 + 8 + 8 + 1 + 1 + 1 + 1 + 1
    /// + (96 * 10) + 32 + 4 + (32 * 10) + 8 + 8 + 8 + 2 + 32 + 8 + 1 + 1 = 1477
    pub const SIZE: usize = 8 + 32 + 32 + 8 + 8 + 1 + 1 + 1 + 1 + 1
        + (96 * 10) + 32 + 4 + (32 * 10) + 8 + 8 + 8 + 2 + 32 + 8 + 1 + 1;

    pub const OUTCOME_UNRESOLVED: u8 = 255;
    pub const OUTCOME_NO: u8 = 0;
    pub const OUTCOME_YES: u8 = 1;
}
