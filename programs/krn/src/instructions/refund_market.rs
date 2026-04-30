use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::KrnError;

/// Refunds a bettor when a market has completely failed to resolve.
/// Only callable when market state is Failed.
pub fn handle_refund_market(ctx: Context<RefundMarket>, _market_id: [u8; 32]) -> Result<()> {
    let market = &ctx.accounts.market;
    require!(
        market.state == MarketState::Failed,
        KrnError::InvalidMarketState
    );

    let commitment = &ctx.accounts.commitment;
    require!(!commitment.claimed, KrnError::NullifierAlreadyUsed);

    let payout = commitment.amount;

    // Transfer refund from pool to bettor
    let market_id = market.market_id;
    let bump = ctx.bumps.market_pool;
    let signer_seeds: &[&[&[u8]]] = &[&[b"pool", market_id.as_ref(), &[bump]]];

    let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.market_pool.key(),
        &ctx.accounts.bettor.key(),
        payout,
    );
    anchor_lang::solana_program::program::invoke_signed(
        &transfer_ix,
        &[
            ctx.accounts.market_pool.to_account_info(),
            ctx.accounts.bettor.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        signer_seeds,
    )?;

    // Mark commitment as claimed to prevent double refund
    let commitment = &mut ctx.accounts.commitment;
    commitment.claimed = true;

    msg!("Refund issued: amount={}", payout);
    Ok(())
}

#[derive(Accounts)]
#[instruction(market_id: [u8; 32])]
pub struct RefundMarket<'info> {
    #[account(
        seeds = [b"market", market_id.as_ref()],
        bump = market.bump,
    )]
    pub market: Account<'info, MarketAccount>,

    #[account(
        mut,
        seeds = [b"commitment", market_id.as_ref(), bettor.key().as_ref()],
        bump = commitment.bump,
    )]
    pub commitment: Account<'info, BetCommitment>,

    /// CHECK: PDA pool vault holding market funds
    #[account(
        mut,
        seeds = [b"pool", market_id.as_ref()],
        bump
    )]
    pub market_pool: UncheckedAccount<'info>,

    #[account(mut)]
    pub bettor: Signer<'info>,

    pub system_program: Program<'info, System>,
}
