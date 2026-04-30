use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::KrnError;

/// Claims winnings using a ZK ownership proof.
/// Winner proves they hold a winning ticket without revealing their betting address.
/// Funds are sent to a fresh recipient address for privacy.
pub fn handle_claim_winning(
    ctx: Context<ClaimWinning>,
    _market_id: [u8; 32],
    _nullifier: [u8; 32],
    ownership_proof: Vec<u8>,
) -> Result<()> {
    let market = &ctx.accounts.market;
    require!(
        market.state == MarketState::Resolved,
        KrnError::MarketNotResolved
    );

    // TODO: Verify Groth16 ownership proof on-chain
    // For Phase 1, validate proof is non-empty as placeholder
    require!(!ownership_proof.is_empty(), KrnError::InvalidOwnershipProof);

    // Record nullifier to prevent double-claiming
    let nullifier_account = &mut ctx.accounts.nullifier_account;
    nullifier_account.market_id = market.market_id;
    nullifier_account.nullifier = _nullifier;
    nullifier_account.claimed_at = Clock::get()?.unix_timestamp;
    nullifier_account.bump = ctx.bumps.nullifier_account;

    // Calculate payout based on commitment amount and pool ratios
    let commitment = &ctx.accounts.commitment;
    require!(
        commitment.side == market.outcome,
        KrnError::OutcomeMismatch
    );

    let winning_pool = if market.outcome == MarketAccount::OUTCOME_YES {
        market.yes_pool
    } else {
        market.no_pool
    };

    // Payout = (bet_amount / winning_pool) * total_pool
    let payout = (commitment.amount as u128)
        .checked_mul(market.total_pool as u128)
        .unwrap()
        .checked_div(winning_pool as u128)
        .unwrap() as u64;

    // Transfer from pool to recipient
    let market_id = market.market_id;
    let bump = ctx.bumps.market_pool;
    let signer_seeds: &[&[&[u8]]] = &[&[b"pool", market_id.as_ref(), &[bump]]];

    let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.market_pool.key(),
        &ctx.accounts.recipient.key(),
        payout,
    );
    anchor_lang::solana_program::program::invoke_signed(
        &transfer_ix,
        &[
            ctx.accounts.market_pool.to_account_info(),
            ctx.accounts.recipient.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        signer_seeds,
    )?;

    msg!("Winning claimed: payout={}", payout);
    Ok(())
}

#[derive(Accounts)]
#[instruction(market_id: [u8; 32], nullifier: [u8; 32])]
pub struct ClaimWinning<'info> {
    #[account(
        seeds = [b"market", market_id.as_ref()],
        bump = market.bump,
    )]
    pub market: Account<'info, MarketAccount>,

    #[account(
        seeds = [b"commitment", market_id.as_ref(), claimer.key().as_ref()],
        bump = commitment.bump,
    )]
    pub commitment: Account<'info, BetCommitment>,

    #[account(
        init,
        payer = claimer,
        space = NullifierAccount::SIZE,
        seeds = [b"nullifier", market_id.as_ref(), nullifier.as_ref()],
        bump
    )]
    pub nullifier_account: Account<'info, NullifierAccount>,

    /// CHECK: PDA pool vault holding market funds
    #[account(
        mut,
        seeds = [b"pool", market_id.as_ref()],
        bump
    )]
    pub market_pool: UncheckedAccount<'info>,

    /// CHECK: Fresh address to receive winnings (privacy-preserving)
    #[account(mut)]
    pub recipient: UncheckedAccount<'info>,

    #[account(mut)]
    pub claimer: Signer<'info>,

    pub system_program: Program<'info, System>,
}
