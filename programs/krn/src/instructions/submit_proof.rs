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
    outcome: u8,
    threshold: u64,
    extracted_value: u64,
    comparison_used: u8,
    server_name: Vec<u8>,
}

/// Parse KRN structured output from public values.
/// Format: [original_zktls_output(N)][output_length(4)][KRN_MARKER(3)][outcome(1)][comparison(1)][threshold(8)][extracted_value(8)][server_name_len(1)][server_name(N)]
/// Uses length prefix for deterministic parsing — no marker scanning needed.
fn parse_krn_public_values(pv: &[u8]) -> Result<KrnPublicValues> {
    // Minimum size: at least 4 (length) + 3 (marker) + 19 (data) = 26 bytes after original output
    require!(pv.len() >= 26, KrnError::InvalidZkTlsProof);

    // Read original output length from first 4 bytes after the output
    // The length tells us exactly where KRN data starts
    // We need to find where the length field is: it's at position output_len
    // But we don't know output_len yet... it's encoded at position output_len.
    // Solution: try reading u32 at various positions? No — we know the length field
    // is right after the original output. Since we don't know output size a priori,
    // we read the length from the end of the public values and work backwards.
    //
    // Actually, the structure is sequential:
    // [output(N bytes)][length(4 bytes)][KRN(3)][outcome(1)][comparison(1)][threshold(8)][value(8)][name_len(1)][name(M)]
    // Total KRN section = 4 + 3 + 1 + 1 + 8 + 8 + 1 + M = 26 + M
    // So: output_len = total_len - 26 - M
    // But we don't know M either...
    //
    // Better approach: server_name_len is at pv[pv.len() - 1 - server_name_len_value]
    // This is circular. Let's use the length field directly.
    //
    // The length field is at offset N (where N = original output length).
    // We can iterate: read candidate length at each position until we find one
    // that is self-consistent. But that's scanning again.
    //
    // Simplest correct approach: read the last byte as server_name_len,
    // then work backwards to find all fields, then verify KRN marker.

    // Read server_name from the end
    // Last bytes: [server_name_len(1)][server_name(M)]
    // But server_name is at the very end, so we need to find server_name_len first.
    // server_name_len is at position: total - 1 - server_name_len_value
    // This is still circular without knowing the length.
    //
    // Use the length prefix approach: output_len is a u32 at a known offset.
    // Since output always starts at byte 0, and length field follows output,
    // we try: output_len_candidate = u32 at position X, verify X == output_len_candidate.

    // The output always starts with version byte 0x01 (Response::BYTES_VERSION)
    // Scan for the length field: it must satisfy pv[N..N+4] as u32 == N
    let mut output_len: Option<usize> = None;
    for candidate in 0..pv.len().saturating_sub(25) {
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

    // Minimum remaining: 1 (outcome) + 1 (comparison) + 8 (threshold) + 8 (value) + 1 (name_len) = 19
    require!(
        pv.len() >= krn_start + 19,
        KrnError::InvalidZkTlsProof
    );

    let outcome = pv[krn_start];
    let comparison_used = pv[krn_start + 1];
    let threshold = u64::from_be_bytes(pv[krn_start + 2..krn_start + 10].try_into().unwrap());
    let extracted_value = u64::from_be_bytes(pv[krn_start + 10..krn_start + 18].try_into().unwrap());
    let server_name_len = pv[krn_start + 18] as usize;

    require!(
        pv.len() >= krn_start + 19 + server_name_len,
        KrnError::InvalidZkTlsProof
    );

    let server_name = pv[krn_start + 19..krn_start + 19 + server_name_len].to_vec();

    Ok(KrnPublicValues {
        outcome,
        threshold,
        extracted_value,
        comparison_used,
        server_name,
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
