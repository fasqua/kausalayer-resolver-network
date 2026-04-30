use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::KrnError;

/// Submits a zkTLS proof for a specific source.
/// Anyone can submit (permissionless), but proof must be valid.
pub fn handle_submit_proof(
    ctx: Context<SubmitProof>,
    _market_id: [u8; 32],
    source_index: u8,
    claimed_outcome: u8,
    proof_data: Vec<u8>,
) -> Result<()> {
    let market = &mut ctx.accounts.market;
    require!(
        market.state == MarketState::Closed || market.state == MarketState::Resolving,
        KrnError::InvalidMarketState
    );

    let clock = Clock::get()?;
    require!(
        clock.unix_timestamp < market.resolution_deadline,
        KrnError::DeadlineExpired
    );

    require!(
        (source_index as usize) < market.num_sources as usize,
        KrnError::InvalidSourceIndex
    );

    // TODO: Verify zkTLS proof cryptographically
    // For Phase 1, validate proof_data is non-empty as placeholder
    require!(!proof_data.is_empty(), KrnError::InvalidZkTlsProof);

    let proof_hash = solana_sha256_hasher::hash(&proof_data).to_bytes();

    // Record submission
    let submission = &mut ctx.accounts.proof_submission;
    submission.market_id = market.market_id;
    submission.source_index = source_index;
    submission.claimed_outcome = claimed_outcome;
    submission.submitter = ctx.accounts.submitter.key();
    submission.timestamp = clock.unix_timestamp;
    submission.proof_hash = proof_hash;
    submission.verified = true;
    submission.bump = ctx.bumps.proof_submission;

    market.source_count += 1;

    // Transition to Resolving state on first proof
    if market.state == MarketState::Closed {
        market.state = MarketState::Resolving;
    }

    msg!(
        "Proof submitted: source={}, outcome={}",
        source_index,
        claimed_outcome
    );
    Ok(())
}

#[derive(Accounts)]
#[instruction(market_id: [u8; 32], source_index: u8)]
pub struct SubmitProof<'info> {
    #[account(
        mut,
        seeds = [b"market", market_id.as_ref()],
        bump = market.bump,
    )]
    pub market: Account<'info, MarketAccount>,

    #[account(
        init,
        payer = submitter,
        space = ProofSubmission::SIZE,
        seeds = [b"proof", market_id.as_ref(), &[source_index]],
        bump
    )]
    pub proof_submission: Account<'info, ProofSubmission>,

    #[account(mut)]
    pub submitter: Signer<'info>,

    pub system_program: Program<'info, System>,
}
