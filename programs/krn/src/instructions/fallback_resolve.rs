use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::KrnError;

/// Manual resolution by authority when automated resolution fails.
/// Only callable after resolution deadline has passed and market is not yet resolved.
pub fn handle_fallback_resolve(
    ctx: Context<FallbackResolve>,
    outcome: u8,
) -> Result<()> {
    let market = &mut ctx.accounts.market;
    require!(
        market.state == MarketState::Closed
            || market.state == MarketState::Resolving
            || market.state == MarketState::Failed,
        KrnError::InvalidMarketState
    );
    require!(
        ctx.accounts.authority.key() == market.authority,
        KrnError::Unauthorized
    );

    let clock = Clock::get()?;
    require!(
        clock.unix_timestamp >= market.resolution_deadline,
        KrnError::DeadlineNotReached
    );
    require!(
        outcome == MarketAccount::OUTCOME_NO || outcome == MarketAccount::OUTCOME_YES,
        KrnError::InvalidBetSide
    );

    market.outcome = outcome;
    market.state = MarketState::Resolved;

    msg!("Market fallback resolved: outcome={}", outcome);
    Ok(())
}

#[derive(Accounts)]
pub struct FallbackResolve<'info> {
    #[account(
        mut,
        seeds = [b"market", market.market_id.as_ref()],
        bump = market.bump,
    )]
    pub market: Account<'info, MarketAccount>,

    pub authority: Signer<'info>,
}
