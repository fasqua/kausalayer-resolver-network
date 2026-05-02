use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;
pub mod errors;

use instructions::*;
use state::SourceConfig;

declare_id!("5qkQX3VaiUni5xLA7HQGbGPCPLajbELoj5QAA2PbnFDK");

#[program]
pub mod krn {
    use super::*;

    pub fn init_market(
        ctx: Context<InitMarket>,
        market_id: [u8; 32],
        close_timestamp: i64,
        resolution_deadline: i64,
        source_configs: Vec<SourceConfig>,
        required_sources: u8,
    ) -> Result<()> {
        instructions::init_market::handle_init_market(
            ctx, market_id, close_timestamp,
            resolution_deadline, source_configs, required_sources,
        )
    }

    pub fn place_bet(
        ctx: Context<PlaceBet>,
        market_id: [u8; 32],
        commitment_hash: [u8; 32],
        side: u8,
        amount: u64,
    ) -> Result<()> {
        instructions::place_bet::handle_place_bet(ctx, market_id, commitment_hash, side, amount)
    }

    pub fn close_market(ctx: Context<CloseMarket>) -> Result<()> {
        instructions::close_market::handle_close_market(ctx)
    }

    pub fn submit_proof(
        ctx: Context<SubmitProof>,
        market_id: [u8; 32],
        source_index: u8,
        claimed_outcome: u8,
        reclaim_proof: ReclaimProof,
    ) -> Result<()> {
        instructions::submit_proof::handle_submit_proof(ctx, market_id, source_index, claimed_outcome, reclaim_proof)
    }

    pub fn aggregate_resolution(ctx: Context<AggregateResolution>) -> Result<()> {
        instructions::aggregate_resolution::handle_aggregate_resolution(ctx)
    }

    pub fn claim_winning(
        ctx: Context<ClaimWinning>,
        market_id: [u8; 32],
        nullifier: [u8; 32],
        ownership_proof: OwnershipProofData,
    ) -> Result<()> {
        instructions::claim_winning::handle_claim_winning(ctx, market_id, nullifier, ownership_proof)
    }

    pub fn fallback_resolve(
        ctx: Context<FallbackResolve>,
        outcome: u8,
    ) -> Result<()> {
        instructions::fallback_resolve::handle_fallback_resolve(ctx, outcome)
    }

    pub fn refund_market(
        ctx: Context<RefundMarket>,
        market_id: [u8; 32],
    ) -> Result<()> {
        instructions::refund_market::handle_refund_market(ctx, market_id)
    }

    pub fn verify_sp1_zktls(
        _ctx: Context<VerifySp1ZkTls>,
        proof_data: SP1ZkTlsProofData,
    ) -> Result<()> {
        instructions::sp1_verifier::verify_sp1_zktls_proof(&proof_data)?;
        msg!("SP1 zkTLS proof verified on-chain");
        msg!("Public values: {} bytes", proof_data.public_values.len());
        msg!("Proof: {} bytes", proof_data.proof.len());
        Ok(())
    }
}