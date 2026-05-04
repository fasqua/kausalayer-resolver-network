use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::KrnError;

/// Creates a new prediction market with source configurations.
/// Caller must specify minimum sources, source configs, timestamps, and deadline.
pub fn handle_init_market(
    ctx: Context<InitMarket>,
    market_id: [u8; 32],
    close_timestamp: i64,
    resolution_deadline: i64,
    source_configs: Vec<SourceConfig>,
    required_sources: u8,
    threshold: u64,
    comparison: u8,
) -> Result<()> {
    require!(
        source_configs.len() >= required_sources as usize,
        KrnError::MinSourcesRequired
    );
    require!(required_sources >= 1, KrnError::MinSourcesRequired);
    require!(source_configs.len() <= 10, KrnError::TooManySources);
    require!(
        close_timestamp < resolution_deadline,
        KrnError::InvalidTimestamps
    );

    // Ensure close_timestamp is in the future (prevents griefing with already-expired markets)
    let clock = Clock::get()?;
    require!(
        close_timestamp > clock.unix_timestamp,
        KrnError::InvalidTimestamps
    );

    let market = &mut ctx.accounts.market;
    market.market_id = market_id;
    market.creator = ctx.accounts.creator.key();
    market.close_timestamp = close_timestamp;
    market.resolution_deadline = resolution_deadline;
    market.state = MarketState::Open;
    market.outcome = MarketAccount::OUTCOME_UNRESOLVED;
    market.source_count = 0;
    market.required_sources = required_sources;
    market.num_sources = source_configs.len() as u8;

    // Copy source configs into fixed-size array
    let mut configs = [SourceConfig {
        domain_hash: [0u8; 32],
        path_hash: [0u8; 32],
        json_path_hash: [0u8; 32],
    }; 10];
    for (i, config) in source_configs.iter().enumerate() {
        configs[i] = config.clone();
    }
    market.source_configs = configs;

    market.commitment_root = [0u8; 32];
    market.commitment_count = 0;
    market.commitment_tree = [[0u8; 32]; 10];
    market.total_pool = 0;
    market.yes_pool = 0;
    market.no_pool = 0;
    market.nullifier_count = 0;
    market.authority = ctx.accounts.creator.key();
    market.threshold = threshold;
    market.comparison = comparison;
    market.bump = ctx.bumps.market;

    msg!("Market initialized: {:?}", market_id);
    Ok(())
}

#[derive(Accounts)]
#[instruction(market_id: [u8; 32])]
pub struct InitMarket<'info> {
    #[account(
        init,
        payer = creator,
        space = MarketAccount::SIZE,
        seeds = [b"market", market_id.as_ref()],
        bump
    )]
    pub market: Account<'info, MarketAccount>,

    #[account(mut)]
    pub creator: Signer<'info>,

    pub system_program: Program<'info, System>,
}
