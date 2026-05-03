use anchor_lang::prelude::*;
use anchor_lang::system_program;
use solana_poseidon::{hashv as poseidon_hashv, Parameters, Endianness};
use crate::state::*;
use crate::errors::KrnError;

/// Depth of the incremental Merkle tree (max 2^10 = 1024 bets per market).
const TREE_DEPTH: usize = 10;

/// Compute Poseidon hash of two 32-byte inputs via native Solana syscall.
/// Uses BN254 curve, big-endian (matches circomlib Poseidon(2)).
fn poseidon_hash_two(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let hash = poseidon_hashv(
        Parameters::Bn254X5,
        Endianness::BigEndian,
        &[left.as_ref(), right.as_ref()],
    ).unwrap();
    hash.to_bytes()
}

/// Pre-computed zero hashes for each level of the Merkle tree.
/// ZERO_HASHES[0] = [0u8; 32]
/// ZERO_HASHES[n] = Poseidon(ZERO_HASHES[n-1], ZERO_HASHES[n-1])
fn compute_zero_hashes() -> [[u8; 32]; TREE_DEPTH] {
    let mut zeros = [[0u8; 32]; TREE_DEPTH];
    for i in 1..TREE_DEPTH {
        zeros[i] = poseidon_hash_two(&zeros[i - 1], &zeros[i - 1]);
    }
    zeros
}

/// Inserts a leaf into the incremental Merkle tree and returns the new root.
/// This follows the standard append-only incremental Merkle tree pattern
/// Uses Poseidon hash via native Solana syscall (no stack overflow).
/// Matches circomlib Poseidon(2) used in ownership.circom.
///
/// Algorithm:
/// - Start with current_hash = leaf
/// - At each level, check if the current index is even or odd at that level
/// - If even: store current_hash in tree[level], pair with zero_hash
/// - If odd: pair with tree[level] (the previously stored sibling)
/// - Move up until root
fn insert_leaf(
    tree: &mut [[u8; 32]; TREE_DEPTH],
    leaf: [u8; 32],
    index: u32,
) -> [u8; 32] {
    let zero_hashes = compute_zero_hashes();
    let mut current_hash = leaf;
    let mut current_index = index;

    for level in 0..TREE_DEPTH {
        if current_index % 2 == 0 {
            // Even index: this node is a left child
            // Store it for future right sibling pairing
            tree[level] = current_hash;
            // Pair with zero hash (right sibling doesn't exist yet)
            current_hash = poseidon_hash_two(&current_hash, &zero_hashes[level]);
        } else {
            // Odd index: this node is a right child
            // Pair with the stored left sibling
            current_hash = poseidon_hash_two(&tree[level], &current_hash);
        }
        current_index /= 2;
    }

    current_hash
}

/// Places a bet on a market with a commitment hash for privacy.
/// The commitment root is computed on-chain using an incremental Poseidon Merkle tree.
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

    // Check tree capacity (max 2^16 = 65536 bets)
    require!(
        market.commitment_count < (1u32 << TREE_DEPTH),
        KrnError::MerkleTreeFull
    );

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

    // Insert commitment into on-chain incremental Merkle tree
    let leaf_index = market.commitment_count;
    let new_root = insert_leaf(
        &mut market.commitment_tree,
        commitment_hash,
        leaf_index,
    );
    market.commitment_root = new_root;
    market.commitment_count += 1;

    msg!("Bet placed: side={}, amount={}, leaf_index={}", side, amount, leaf_index);
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

