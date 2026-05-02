use anchor_lang::prelude::*;
use groth16_solana::groth16::{Groth16Verifyingkey, Groth16Verifier};

// Auto-generated verifying key from ownership.circom
// 5 public inputs: market_id, resolved_outcome, nullifier, commitment_root, amount
// Auto-generated from verification_key.json
// Circuit: ownership.circom (5 public inputs)

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
    19, 183, 5, 239, 188, 93, 250, 149, 72, 147, 215, 130, 79, 98, 136, 21,
    179, 88, 239, 162, 236, 115, 42, 168, 177, 8, 116, 68, 245, 0, 10, 15,
    48, 64, 137, 152, 245, 145, 72, 60, 156, 144, 217, 76, 17, 15, 175, 152,
    208, 198, 139, 148, 187, 119, 104, 107, 119, 27, 111, 63, 251, 157, 131, 137,
    18, 186, 208, 136, 249, 31, 126, 47, 71, 222, 254, 134, 205, 75, 234, 233,
    3, 124, 162, 44, 166, 178, 44, 243, 176, 105, 116, 181, 223, 121, 24, 212,
    30, 191, 133, 22, 214, 132, 102, 155, 93, 27, 241, 103, 26, 217, 220, 180,
    224, 15, 158, 224, 171, 74, 171, 121, 62, 176, 220, 242, 41, 230, 147, 254,
];

pub const VK_IC: [[u8; 64]; 6] = [
    [
        12, 239, 192, 141, 252, 180, 23, 75, 37, 73, 120, 78, 252, 63, 34, 203,
        109, 171, 101, 252, 69, 36, 83, 200, 131, 93, 199, 254, 192, 79, 131, 177,
        44, 88, 249, 42, 183, 178, 217, 109, 19, 75, 184, 97, 184, 235, 13, 8,
        105, 224, 97, 27, 212, 48, 118, 14, 21, 53, 103, 86, 84, 35, 128, 216,
    ],
    [
        7, 69, 36, 246, 128, 205, 9, 106, 98, 98, 48, 92, 54, 136, 61, 163,
        254, 217, 219, 64, 226, 175, 59, 198, 253, 55, 38, 78, 19, 239, 238, 74,
        27, 200, 195, 253, 8, 164, 212, 146, 38, 1, 49, 220, 103, 22, 251, 165,
        45, 175, 167, 130, 153, 210, 21, 201, 176, 159, 210, 53, 55, 79, 17, 192,
    ],
    [
        18, 175, 131, 96, 178, 74, 121, 181, 58, 247, 60, 232, 242, 207, 157, 103,
        242, 213, 109, 224, 189, 131, 4, 153, 126, 129, 21, 38, 35, 3, 135, 1,
        31, 196, 76, 21, 33, 72, 9, 205, 143, 81, 188, 20, 0, 155, 137, 97,
        233, 190, 37, 198, 103, 176, 246, 59, 253, 103, 48, 84, 170, 171, 214, 118,
    ],
    [
        32, 140, 62, 55, 188, 73, 24, 188, 41, 51, 134, 13, 105, 108, 61, 201,
        188, 243, 3, 131, 123, 239, 27, 33, 74, 243, 137, 53, 104, 104, 18, 240,
        33, 193, 157, 84, 107, 32, 179, 80, 129, 37, 79, 63, 75, 5, 104, 165,
        37, 162, 144, 49, 10, 108, 49, 186, 63, 64, 11, 98, 251, 225, 177, 159,
    ],
    [
        41, 177, 7, 214, 228, 60, 240, 116, 43, 22, 46, 145, 150, 225, 183, 128,
        138, 186, 106, 117, 57, 32, 241, 236, 93, 207, 75, 245, 14, 233, 70, 203,
        6, 17, 239, 13, 8, 72, 137, 103, 241, 228, 131, 187, 141, 57, 141, 114,
        129, 5, 62, 5, 180, 142, 217, 226, 184, 76, 242, 203, 253, 14, 1, 108,
    ],
    [
        18, 222, 68, 171, 184, 97, 220, 190, 140, 34, 154, 95, 186, 49, 168, 82,
        92, 140, 125, 2, 193, 156, 179, 252, 246, 76, 152, 30, 106, 15, 36, 7,
        31, 252, 94, 158, 104, 18, 203, 106, 217, 31, 199, 112, 182, 190, 174, 106,
        36, 115, 58, 115, 156, 87, 68, 68, 249, 138, 201, 119, 49, 152, 213, 148,
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
    /// Proof point B (G2, 128 bytes big-endian)
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
    // Public input [0]: market_id
    require!(
        proof.public_inputs[0] == *market_id,
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
