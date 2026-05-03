use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::KrnError;
use crate::instructions::sp1_verifier::{SP1ZkTlsProofData, verify_sp1_zktls_proof};
use solana_sha256_hasher::hashv as sha256;

/// KRN marker in public values: "KRN" = [0x4B, 0x52, 0x4E]
const KRN_MARKER: [u8; 3] = [0x4B, 0x52, 0x4E];

/// Submits a zkTLS proof for a specific source.
/// Anyone can submit (permissionless). Proof is verified via SP1 Groth16 on-chain.
/// Outcome is extracted from the proof's public values — not from user input.
pub fn handle_submit_proof(
    ctx: Context<SubmitProof>,
    _market_id: [u8; 32],
    source_index: u8,
    sp1_proof: SP1ZkTlsProofData,
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

    // Verify SP1 Groth16 proof on-chain
    verify_sp1_zktls_proof(&sp1_proof)?;

    // Parse KRN structured output from public values
    let pv = &sp1_proof.public_values;
    let krn_data = parse_krn_public_values(pv)?;

    // Validate threshold matches market config
    require!(
        krn_data.threshold == market.threshold,
        KrnError::ThresholdMismatch
    );

    // Validate comparison matches market config
    require!(
        krn_data.comparison_used == market.comparison,
        KrnError::ThresholdMismatch
    );

    // Validate server domain matches source config
    let domain_hash = sha256(&[&krn_data.server_name]).to_bytes();
    let source_config = &market.source_configs[source_index as usize];
    require!(
        domain_hash == source_config.domain_hash,
        KrnError::DomainMismatch
    );

    // Validate json_path matches source config
    let path_hash = sha256(&[&krn_data.json_path]).to_bytes();
    require!(
        path_hash == source_config.json_path_hash,
        KrnError::DomainMismatch
    );

    // Extract outcome from proof (determined inside zkVM, trustless)
    let claimed_outcome = krn_data.outcome;
    require!(
        claimed_outcome == MarketAccount::OUTCOME_NO || claimed_outcome == MarketAccount::OUTCOME_YES,
        KrnError::InvalidZkTlsProof
    );

    // Record submission
    let proof_hash = sha256(&[pv]).to_bytes();
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
        "SP1 zkTLS proof submitted: source={}, outcome={}, value={}",
        source_index,
        claimed_outcome,
        krn_data.extracted_value
    );
    Ok(())
}

/// Parsed KRN data from SP1 proof public values
struct KrnPublicValues {
    _version: u8,
    outcome: u8,
    threshold: u64,
    extracted_value: u64,
    comparison_used: u8,
    server_name: Vec<u8>,
    json_path: Vec<u8>,
}

/// Parse KRN structured output from public values.
/// v2 format: [original_zktls_output(N)][output_length(4)][KRN_MARKER(3)][version(1)][outcome(1)][comparison(1)][threshold(8)][extracted_value(8)][server_name_len(1)][server_name(S)][json_path_len(1)][json_path(P)]
/// Uses length prefix for deterministic parsing — no marker scanning needed.
fn parse_krn_public_values(pv: &[u8]) -> Result<KrnPublicValues> {
    // Minimum size: at least 4 (length) + 3 (marker) + 1 (version) + 19 (data) + 1 (path_len) = 28 bytes after original output
    require!(pv.len() >= 28, KrnError::InvalidZkTlsProof);

    // Scan for the length field: it must satisfy pv[N..N+4] as u32 == N
    // Then verify KRN marker follows at N+4
    let mut output_len: Option<usize> = None;
    for candidate in 0..pv.len().saturating_sub(27) {
        let len_bytes: [u8; 4] = pv[candidate..candidate + 4].try_into().unwrap();
        let len_val = u32::from_be_bytes(len_bytes) as usize;
        if len_val == candidate {
            // Verify KRN marker follows
            if candidate + 4 + 3 <= pv.len()
                && pv[candidate + 4] == KRN_MARKER[0]
                && pv[candidate + 4 + 1] == KRN_MARKER[1]
                && pv[candidate + 4 + 2] == KRN_MARKER[2]
            {
                output_len = Some(candidate);
                break;
            }
        }
    }

    let output_len = output_len.ok_or_else(|| error!(KrnError::InvalidZkTlsProof))?;
    let krn_start = output_len + 4 + 3; // skip length + marker

    // Read version byte
    require!(
        pv.len() >= krn_start + 1,
        KrnError::InvalidZkTlsProof
    );
    let _version = pv[krn_start];
    require!(_version == 0x02, KrnError::InvalidZkTlsProof);

    let data_start = krn_start + 1; // skip version

    // Minimum remaining: 1 (outcome) + 1 (comparison) + 8 (threshold) + 8 (value) + 1 (name_len) + 1 (path_len) = 20
    require!(
        pv.len() >= data_start + 20,
        KrnError::InvalidZkTlsProof
    );

    let outcome = pv[data_start];
    let comparison_used = pv[data_start + 1];
    let threshold = u64::from_be_bytes(pv[data_start + 2..data_start + 10].try_into().unwrap());
    let extracted_value = u64::from_be_bytes(pv[data_start + 10..data_start + 18].try_into().unwrap());
    let server_name_len = pv[data_start + 18] as usize;

    require!(
        pv.len() >= data_start + 19 + server_name_len,
        KrnError::InvalidZkTlsProof
    );

    let server_name = pv[data_start + 19..data_start + 19 + server_name_len].to_vec();

    // Parse json_path after server_name
    let path_offset = data_start + 19 + server_name_len;
    require!(
        pv.len() >= path_offset + 1,
        KrnError::InvalidZkTlsProof
    );
    let json_path_len = pv[path_offset] as usize;

    require!(
        pv.len() >= path_offset + 1 + json_path_len,
        KrnError::InvalidZkTlsProof
    );

    let json_path = pv[path_offset + 1..path_offset + 1 + json_path_len].to_vec();

    Ok(KrnPublicValues {
        _version,
        outcome,
        threshold,
        extracted_value,
        comparison_used,
        server_name,
        json_path,
    })
}

#[derive(Accounts)]
#[instruction(market_id: [u8; 32], source_index: u8)]
pub struct SubmitProof<'info> {
    #[account(
        mut,
        seeds = [b"market", market_id.as_ref()],
        bump = market.bump,
    )]
    pub market: Box<Account<'info, MarketAccount>>,

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
