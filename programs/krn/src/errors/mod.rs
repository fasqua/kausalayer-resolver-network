use anchor_lang::prelude::*;

#[error_code]
pub enum KrnError {
    #[msg("Minimum 3 sources required")]
    MinSourcesRequired,
    #[msg("Close timestamp must be before resolution deadline")]
    InvalidTimestamps,
    #[msg("Too many source configs (max 10)")]
    TooManySources,
    #[msg("Market is not in the expected state")]
    InvalidMarketState,
    #[msg("Resolution deadline has passed")]
    DeadlineExpired,
    #[msg("Resolution deadline has not passed yet")]
    DeadlineNotReached,
    #[msg("Source index out of bounds")]
    InvalidSourceIndex,
    #[msg("Domain hash does not match source config")]
    DomainMismatch,
    #[msg("Invalid zkTLS proof")]
    InvalidZkTlsProof,
    #[msg("Invalid ownership proof")]
    InvalidOwnershipProof,
    #[msg("No majority reached for resolution")]
    NoMajority,
    #[msg("Not enough proofs submitted")]
    InsufficientProofs,
    #[msg("Nullifier already used (double-claim attempt)")]
    NullifierAlreadyUsed,
    #[msg("Outcome does not match resolved outcome")]
    OutcomeMismatch,
    #[msg("Market has not been resolved yet")]
    MarketNotResolved,
    #[msg("Invalid bet side (must be 0 or 1)")]
    InvalidBetSide,
    #[msg("Bet amount must be greater than zero")]
    ZeroBetAmount,
    #[msg("Unauthorized caller")]
    Unauthorized,
}
