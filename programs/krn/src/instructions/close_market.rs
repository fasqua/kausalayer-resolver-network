use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::KrnError;

/// Closes betting on a market after the close timestamp has passed.
/// Anyone can call this once the deadline is reached.
pub fn handle_close_market(ctx: Context<CloseMarket>) -> Result<()> {
    let market = &mut ctx.accounts.market;
    require!(market.state == MarketState::Open, KrnError::InvalidMarketState);

    let clock = Clock::get()?;
    require!(
        clock.unix_timestamp >= market.close_timestamp,
        KrnError::DeadlineNotReached
    );

    market.state = MarketState::Closed;
    msg!("Market closed, awaiting resolution");
    Ok(())
}

#[derive(Accounts)]
pub struct CloseMarket<'info> {
    #[account(
        mut,
        seeds = [b"market", market.market_id.as_ref()],
        bump = market.bump,
    )]
    pub market: Account<'info, MarketAccount>,

    pub caller: Signer<'info>,
}
