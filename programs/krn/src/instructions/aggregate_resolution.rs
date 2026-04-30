use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::KrnError;

/// Triggers aggregation after enough proofs have been submitted.
/// Counts votes per outcome and resolves if majority is reached.
pub fn handle_aggregate_resolution(ctx: Context<AggregateResolution>) -> Result<()> {
    let market = &mut ctx.accounts.market;
    require!(
        market.state == MarketState::Resolving,
        KrnError::InvalidMarketState
    );
    require!(
        market.source_count >= market.required_sources,
        KrnError::InsufficientProofs
    );

    // Count votes from proof submissions via remaining accounts
    let mut yes_count: u8 = 0;
    let mut no_count: u8 = 0;

    for account_info in ctx.remaining_accounts.iter() {
        let data = account_info.try_borrow_data()?;
        // Skip 8-byte discriminator, then read proof fields
        if data.len() >= ProofSubmission::SIZE - 8 {
            // claimed_outcome is at offset 8 + 32 + 1 = 41 (after discriminator + market_id + source_index)
            let claimed_outcome = data[8 + 32 + 1];
            let verified = data[8 + 32 + 1 + 1 + 32 + 8 + 32] != 0;
            if verified {
                match claimed_outcome {
                    0 => no_count += 1,
                    1 => yes_count += 1,
                    _ => {}
                }
            }
        }
    }

    let majority_threshold = (market.source_count + 1) / 2;

    if yes_count >= majority_threshold {
        market.outcome = MarketAccount::OUTCOME_YES;
        market.state = MarketState::Resolved;
        msg!("Market resolved: YES (votes: {}/{})", yes_count, market.source_count);
    } else if no_count >= majority_threshold {
        market.outcome = MarketAccount::OUTCOME_NO;
        market.state = MarketState::Resolved;
        msg!("Market resolved: NO (votes: {}/{})", no_count, market.source_count);
    } else {
        return Err(KrnError::NoMajority.into());
    }

    Ok(())
}

#[derive(Accounts)]
pub struct AggregateResolution<'info> {
    #[account(
        mut,
        seeds = [b"market", market.market_id.as_ref()],
        bump = market.bump,
    )]
    pub market: Account<'info, MarketAccount>,

    pub caller: Signer<'info>,
}
