use anchor_lang::prelude::*;
use groth16_solana::groth16::{Groth16Verifyingkey, Groth16Verifier};

// Auto-generated verifying key from ownership.circom
// 5 public inputs: market_id, resolved_outcome, nullifier, commitment_root, amount
// Circuit: ownership.circom (depth-10 Merkle tree, 5 public inputs)
// G2 points in EIP-197 format: [x_im, x_re, y_im, y_re]

pub const VK_ALPHA_G1: [u8; 64] = [
    8, 167, 223, 179, 147, 229, 89, 139, 89, 184, 40, 121, 210, 116, 49, 118,
    32, 245, 35, 87, 86, 166, 82, 36, 173, 86, 43, 74, 83, 133, 169, 109,
    47, 235, 134, 44, 195, 66, 63, 155, 166, 239, 78, 111, 146, 238, 204, 201,
    122, 168, 227, 93, 95, 197, 119, 53, 172, 29, 226, 17, 166, 111, 37, 18,
];

pub const VK_BETA_G2: [u8; 128] = [
    25, 41, 61, 154, 18, 131, 54, 47, 186, 68, 81, 11, 139, 252, 125, 49,
    229, 138, 228, 107, 173, 117, 103, 53, 49, 163, 217, 137, 220, 149, 224, 199,
    32, 211, 200, 205, 218, 134, 117, 189, 65, 79, 191, 247, 87, 238, 252, 196,
    3, 49, 170, 251, 158, 15, 140, 187, 243, 78, 142, 4, 44, 100, 234, 190,
    47, 237, 57, 98, 159, 46, 143, 249, 129, 214, 91, 138, 82, 236, 183, 220,
    20, 149, 249, 47, 99, 159, 168, 34, 227, 6, 194, 37, 227, 113, 105, 183,
    20, 100, 224, 71, 53, 103, 230, 44, 51, 82, 107, 228, 139, 170, 219, 153,
    140, 0, 67, 131, 124, 69, 167, 159, 236, 167, 84, 232, 144, 114, 132, 199,
];

pub const VK_GAMMA_G2: [u8; 128] = [
    25, 142, 147, 147, 146, 13, 72, 58, 114, 96, 191, 183, 49, 251, 93, 37,
    241, 170, 73, 51, 53, 169, 231, 18, 151, 228, 133, 183, 174, 243, 18, 194,
    24, 0, 222, 239, 18, 31, 30, 118, 66, 106, 0, 102, 94, 92, 68, 121,
    103, 67, 34, 212, 247, 94, 218, 221, 70, 222, 189, 92, 217, 146, 246, 237,
    9, 6, 137, 208, 88, 95, 240, 117, 236, 158, 153, 173, 105, 12, 51, 149,
    188, 75, 49, 51, 112, 179, 142, 243, 85, 172, 218, 220, 209, 34, 151, 91,
    18, 200, 94, 165, 219, 140, 109, 235, 74, 171, 113, 128, 141, 203, 64, 143,
    227, 209, 231, 105, 12, 67, 211, 123, 76, 230, 204, 1, 102, 250, 125, 170,
];

pub const VK_DELTA_G2: [u8; 128] = [
    37, 108, 116, 238, 126, 30, 120, 221, 209, 180, 227, 67, 180, 111, 64, 6,
    250, 41, 60, 22, 146, 178, 178, 57, 143, 58, 61, 109, 222, 202, 119, 119,
    38, 102, 18, 158, 150, 136, 74, 141, 115, 221, 82, 84, 142, 223, 144, 251,
    77, 98, 182, 236, 112, 241, 129, 164, 2, 58, 152, 163, 143, 118, 250, 1,
    18, 13, 107, 155, 178, 45, 114, 180, 73, 232, 185, 66, 1, 197, 36, 97,
    229, 242, 9, 181, 254, 114, 0, 33, 165, 15, 152, 124, 240, 157, 107, 44,
    9, 24, 10, 174, 212, 177, 197, 99, 22, 95, 250, 142, 137, 194, 138, 32,
    239, 100, 108, 80, 95, 252, 205, 161, 145, 93, 105, 52, 164, 112, 201, 234,
];

pub const VK_IC: [[u8; 64]; 6] = [
    [
        24, 247, 105, 103, 204, 231, 157, 196, 75, 28, 60, 232, 24, 103, 72, 110,
        94, 136, 120, 30, 30, 10, 21, 108, 145, 49, 44, 210, 73, 213, 39, 86,
        37, 30, 211, 207, 194, 82, 67, 62, 183, 56, 168, 191, 91, 9, 175, 70,
        55, 247, 27, 233, 141, 223, 79, 231, 48, 20, 91, 25, 21, 137, 16, 85,
    ],
    [
        23, 195, 209, 49, 225, 175, 44, 21, 141, 98, 189, 244, 143, 80, 181, 229,
        205, 215, 122, 229, 184, 88, 31, 182, 17, 78, 255, 143, 23, 202, 176, 210,
        19, 244, 151, 226, 139, 115, 78, 151, 182, 22, 185, 12, 165, 220, 220, 4,
        175, 27, 179, 72, 141, 42, 169, 96, 202, 22, 145, 241, 252, 248, 217, 183,
    ],
    [
        45, 229, 38, 64, 244, 104, 37, 184, 154, 18, 69, 126, 253, 85, 80, 204,
        60, 95, 65, 143, 101, 14, 186, 181, 136, 219, 250, 215, 136, 126, 87, 8,
        21, 84, 123, 173, 245, 118, 106, 217, 20, 10, 69, 46, 132, 171, 29, 110,
        65, 253, 223, 234, 93, 168, 187, 135, 20, 164, 75, 21, 122, 90, 80, 199,
    ],
    [
        18, 48, 142, 18, 161, 62, 35, 214, 92, 105, 62, 241, 51, 80, 144, 146,
        203, 167, 140, 37, 52, 95, 22, 156, 254, 131, 89, 97, 195, 225, 12, 143,
        48, 2, 51, 229, 194, 204, 171, 148, 24, 4, 62, 157, 162, 98, 145, 192,
        222, 240, 11, 149, 63, 7, 131, 209, 164, 233, 178, 10, 251, 24, 158, 18,
    ],
    [
        46, 212, 49, 252, 245, 193, 149, 56, 188, 209, 252, 199, 132, 215, 29, 2,
        56, 236, 74, 226, 103, 163, 138, 236, 154, 141, 80, 51, 223, 188, 147, 68,
        3, 245, 130, 47, 187, 189, 146, 1, 71, 62, 54, 153, 23, 224, 250, 227,
        210, 79, 18, 253, 70, 5, 127, 38, 224, 108, 114, 15, 226, 197, 69, 42,
    ],
    [
        31, 138, 17, 173, 212, 132, 207, 31, 17, 18, 100, 76, 160, 157, 67, 109,
        192, 110, 118, 196, 152, 203, 190, 26, 137, 83, 19, 202, 221, 165, 129, 218,
        10, 75, 191, 192, 43, 138, 65, 19, 105, 224, 130, 97, 26, 42, 41, 102,
        113, 170, 148, 103, 39, 40, 138, 9, 30, 219, 117, 157, 14, 184, 76, 88,
    ],
];


pub const OWNERSHIP_VK: Groth16Verifyingkey = Groth16Verifyingkey {
    nr_pubinputs: 5,
    vk_alpha_g1: VK_ALPHA_G1,
    vk_beta_g2: VK_BETA_G2,
    vk_gamme_g2: VK_GAMMA_G2,
    vk_delta_g2: VK_DELTA_G2,
    vk_ic: &VK_IC,
};


/// Ownership proof data submitted by the claimer.
/// Client pre-processes snarkjs proof into this format.
/// proof_a must be negated by client before submission.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct OwnershipProofData {
    /// Negated proof point A (G1, 64 bytes big-endian)
    pub proof_a: [u8; 64],
    /// Proof point B (G2, 128 bytes EIP-197: [x_im, x_re, y_im, y_re])
    pub proof_b: [u8; 128],
    /// Proof point C (G1, 64 bytes big-endian)
    pub proof_c: [u8; 64],
    /// Public inputs: [market_id, resolved_outcome, nullifier, commitment_root, amount]
    /// Each 32 bytes big-endian
    pub public_inputs: [[u8; 32]; 5],
}

/// Verifies a Groth16 ownership proof on-chain using alt_bn128 syscalls.
/// Returns Ok(()) if proof is valid, error otherwise.
pub fn verify_ownership_proof(proof: &OwnershipProofData) -> Result<()> {
    let mut verifier = Groth16Verifier::new(
        &proof.proof_a,
        &proof.proof_b,
        &proof.proof_c,
        &proof.public_inputs,
        &OWNERSHIP_VK,
    )
    .map_err(|_| error!(crate::errors::KrnError::InvalidOwnershipProof))?;

    verifier
        .verify()
        .map_err(|_| error!(crate::errors::KrnError::InvalidOwnershipProof))?;

    Ok(())
}

/// BN254 scalar field prime (r) used by circom/snarkjs.
/// = 21888242871839275222246405745257275088548364400416034343698204186575808495617
const BN254_SCALAR_PRIME: [u8; 32] = [
    0x30, 0x64, 0x4e, 0x72, 0xe1, 0x31, 0xa0, 0x29,
    0xb8, 0x50, 0x45, 0xb6, 0x81, 0x81, 0x58, 0x5d,
    0x28, 0x33, 0xe8, 0x48, 0x79, 0xb9, 0x70, 0x91,
    0x43, 0xe1, 0xf5, 0x93, 0xf0, 0x00, 0x00, 0x01,
];

/// Reduces a 32-byte big-endian value modulo the BN254 scalar field prime.
/// If value >= prime, subtracts prime (may need up to 3 subtractions since 2^256 / prime < 4).
fn reduce_mod_bn254(value: &[u8; 32]) -> [u8; 32] {
    let mut result = *value;
    while ge_bytes(&result, &BN254_SCALAR_PRIME) {
        result = sub_bytes(&result, &BN254_SCALAR_PRIME);
    }
    result
}

/// Compare two 32-byte big-endian values: returns true if a >= b
fn ge_bytes(a: &[u8; 32], b: &[u8; 32]) -> bool {
    for i in 0..32 {
        if a[i] > b[i] { return true; }
        if a[i] < b[i] { return false; }
    }
    true // equal
}

/// Subtract two 32-byte big-endian values: a - b (assumes a >= b)
fn sub_bytes(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let mut result = [0u8; 32];
    let mut borrow: u16 = 0;
    for i in (0..32).rev() {
        let diff = (a[i] as u16).wrapping_sub(b[i] as u16).wrapping_sub(borrow);
        result[i] = diff as u8;
        borrow = if diff > 0xFF { 1 } else { 0 };
    }
    result
}

/// Validates that the proof's public inputs match the on-chain market state.
/// This prevents using a valid proof generated for a different market/outcome.
pub fn validate_ownership_public_inputs(
    proof: &OwnershipProofData,
    market_id: &[u8; 32],
    outcome: u8,
    nullifier: &[u8; 32],
    commitment_root: &[u8; 32],
    amount: u64,
) -> Result<()> {
    // Public input [0]: market_id (reduced mod BN254 scalar field prime)
    // Circuit auto-reduces inputs mod prime, so sha256 market_id (256-bit)
    // may differ from the circuit's field element if sha256 > prime.
    // We must reduce on-chain to match.
    let market_id_reduced = reduce_mod_bn254(market_id);
    require!(
        proof.public_inputs[0] == market_id_reduced,
        crate::errors::KrnError::PublicInputMismatch
    );

    // Public input [1]: resolved_outcome (u8 as 32-byte big-endian field element)
    let mut expected_outcome = [0u8; 32];
    expected_outcome[31] = outcome;
    require!(
        proof.public_inputs[1] == expected_outcome,
        crate::errors::KrnError::PublicInputMismatch
    );

    // Public input [2]: nullifier
    require!(
        proof.public_inputs[2] == *nullifier,
        crate::errors::KrnError::PublicInputMismatch
    );

    // Public input [3]: commitment_root
    require!(
        proof.public_inputs[3] == *commitment_root,
        crate::errors::KrnError::PublicInputMismatch
    );

    // Public input [4]: amount (u64 as 32-byte big-endian field element)
    let mut expected_amount = [0u8; 32];
    expected_amount[24..32].copy_from_slice(&amount.to_be_bytes());
    require!(
        proof.public_inputs[4] == expected_amount,
        crate::errors::KrnError::PublicInputMismatch
    );

    Ok(())
}
