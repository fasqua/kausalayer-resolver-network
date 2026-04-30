use anchor_lang::prelude::*;
use anchor_lang::system_program;
use crate::state::*;
use crate::errors::KrnError;

/// Places a bet on a market with a commitment hash for privacy.
pub fn handle_place_bet(
    ctx: Context<PlaceBet>,
    _market_id: [u8; 32],
    commitment_hash: [u8; 32],
    side: u8,
    amount: u64,
) -> Result<()> {
    require!(
        side == MarketAccount::OUTCOME_NO || side == MarketAccount::OUTCOME_YES,
        KrnError::InvalidBetSide
    );
    require!(amount > 0, KrnError::ZeroBetAmount);

    let market = &mut ctx.accounts.market;
    require!(market.state == MarketState::Open, KrnError::InvalidMarketState);

    // Transfer SOL from bettor to market pool
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.bettor.to_account_info(),
                to: ctx.accounts.market_pool.to_account_info(),
            },
        ),
        amount,
    )?;

    // Update market pools
    market.total_pool = market.total_pool.checked_add(amount).unwrap();
    if side == MarketAccount::OUTCOME_YES {
        market.yes_pool = market.yes_pool.checked_add(amount).unwrap();
    } else {
        market.no_pool = market.no_pool.checked_add(amount).unwrap();
    }

    // Record commitment
    let commitment = &mut ctx.accounts.commitment;
    commitment.market_id = market.market_id;
    commitment.commitment_hash = commitment_hash;
    commitment.side = side;
    commitment.amount = amount;
    commitment.claimed = false;
    commitment.bump = ctx.bumps.commitment;

    msg!("Bet placed: side={}, amount={}", side, amount);
    Ok(())
}

#[derive(Accounts)]
#[instruction(market_id: [u8; 32])]
pub struct PlaceBet<'info> {
    #[account(
        mut,
        seeds = [b"market", market_id.as_ref()],
        bump = market.bump,
    )]
    pub market: Account<'info, MarketAccount>,

    #[account(
        init,
        payer = bettor,
        space = BetCommitment::SIZE,
        seeds = [b"commitment", market_id.as_ref(), bettor.key().as_ref()],
        bump
    )]
    pub commitment: Account<'info, BetCommitment>,

    /// CHECK: PDA used as pool vault for holding funds
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
