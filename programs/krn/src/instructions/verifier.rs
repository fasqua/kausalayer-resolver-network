use anchor_lang::prelude::*;
use solana_keccak_hasher::hashv as keccak256;
use solana_secp256k1_recover::secp256k1_recover;

/// Reclaim proof data for on-chain verification.
/// Client pre-computes the identifier from claim data.
/// On-chain only needs to verify the ECDSA signature.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ReclaimProof {
    /// Pre-computed claim identifier (keccak256 of provider+params+context)
    pub identifier: [u8; 32],
    /// Claim owner as Ethereum address (20 bytes)
    pub owner: [u8; 20],
    /// Unix timestamp of the proof
    pub timestamp_s: u64,
    /// Epoch number
    pub epoch: u8,
    /// ECDSA signature (64 bytes: r[32] + s[32])
    pub signature: [u8; 64],
    /// Recovery ID (0 or 1)
    pub recovery_id: u8,
}

/// Known Reclaim attestor address (Ethereum format, lowercase).
/// Source: wss://attestor.reclaimprotocol.org:444/ws
pub const RECLAIM_ATTESTOR: [u8; 20] = [
    0x24, 0x48, 0x97, 0x57, 0x23, 0x68, 0xea, 0xdf, 0x65, 0xbf,
    0xbc, 0x5a, 0xec, 0x98, 0xd8, 0xe5, 0x44, 0x3a, 0x90, 0x72,
];

/// Verifies a Reclaim zkTLS proof on-chain.
///
/// The client pre-computes the identifier from (provider, parameters, context).
/// On-chain we reconstruct the signed message and verify the ECDSA signature
/// matches a known Reclaim attestor.
///
/// Signed message format (Reclaim SDK):
///   "{identifier_hex}\n{owner_hex}\n{timestampS}\n{epoch}"
///
/// Then Ethereum personal_sign prefix is applied before keccak256.
pub fn verify_reclaim_proof(proof: &ReclaimProof) -> Result<bool> {
    // Step 1: Reconstruct the signed message
    let identifier_hex = hex_with_prefix(&proof.identifier);
    let owner_hex = hex_with_prefix(&proof.owner);

    let signed_message = format!(
        "{}\n{}\n{}\n{}",
        identifier_hex, owner_hex, proof.timestamp_s, proof.epoch
    );

    // Step 2: Apply Ethereum personal_sign prefix
    let msg_bytes = signed_message.as_bytes();
    let prefix = format!("\x19Ethereum Signed Message:\n{}", msg_bytes.len());
    let mut prefixed = Vec::with_capacity(prefix.len() + msg_bytes.len());
    prefixed.extend_from_slice(prefix.as_bytes());
    prefixed.extend_from_slice(msg_bytes);

    // Step 3: Hash with keccak256
    let message_hash = keccak256(&[&prefixed]);

    // Step 4: Recover public key from signature
    let recovered_pubkey = secp256k1_recover(
        &message_hash.0,
        proof.recovery_id,
        &proof.signature,
    )
    .map_err(|_| error!(crate::errors::KrnError::InvalidZkTlsProof))?;

    // Step 5: Derive Ethereum address = keccak256(pubkey)[12..32]
    let pubkey_hash = keccak256(&[&recovered_pubkey.0]);
    let mut recovered_address = [0u8; 20];
    recovered_address.copy_from_slice(&pubkey_hash.0[12..32]);

    // Step 6: Check recovered address matches known attestor
    Ok(recovered_address == RECLAIM_ATTESTOR)
}

/// Convert bytes to 0x-prefixed lowercase hex string
fn hex_with_prefix(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(2 + bytes.len() * 2);
    s.push_str("0x");
    for &b in bytes {
        s.push(HEX_CHARS[(b >> 4) as usize] as char);
        s.push(HEX_CHARS[(b & 0x0f) as usize] as char);
    }
    s
}

const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
