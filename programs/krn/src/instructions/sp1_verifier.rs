use anchor_lang::prelude::*;
use groth16_solana::groth16::{Groth16Verifier, Groth16Verifyingkey};
use ark_bn254::{Fq, G1Affine};
use ark_ff::PrimeField;
use ark_serialize::CanonicalSerialize;
use sha2::{Digest, Sha256};

// ============================================================
// KRN SP1 v6 Verifier
// Uses exact same decompression/negation logic as sp1-solana v0.1.0
// Only difference: 5 public inputs instead of 2, and v6 proof layout
// ============================================================

/// Compressed Groth16 VK for SP1 v6.1.0 (492 bytes)
const COMPRESSED_VK: &[u8] = include_bytes!("../vk/groth16_vk_v6.bin");

/// SHA256(compressed_vk)[0..4]
const VK_HASH_PREFIX: [u8; 4] = [0x43, 0x88, 0xa2, 0x1c];

/// SP1 v6.1.0 recursion VK merkle root
const SP1_V6_VK_ROOT: [u8; 32] = [
    0x00, 0x2f, 0x85, 0x0e, 0xe9, 0x98, 0x97, 0x4d,
    0x6c, 0xc0, 0x0e, 0x50, 0xcd, 0x08, 0x14, 0xb0,
    0x98, 0xc0, 0x5b, 0xfa, 0xde, 0x46, 0x6d, 0x28,
    0x57, 0x32, 0x40, 0xd0, 0x57, 0xf2, 0x53, 0x52,
];

/// KRN zkTLS program vkey hash
pub const KRN_ZKTLS_VKEY_HASH: &str =
    "0x00a52ad84180de09af0c46fdcc8396cd2ac13d73ba6df52ca7e59f29dcb8a9e4";

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SP1ZkTlsProofData {
    pub proof: Vec<u8>,
    pub public_values: Vec<u8>,
}

pub fn verify_sp1_zktls_proof(proof_data: &SP1ZkTlsProofData) -> Result<()> {
    let proof = &proof_data.proof;

    // 1. Validate proof length: 4 + 32 + 32 + 32 + 256 = 356
    require!(proof.len() == 356, crate::errors::KrnError::InvalidZkTlsProof);

    // 2. Verify VK hash prefix
    require!(proof[0..4] == VK_HASH_PREFIX, crate::errors::KrnError::InvalidZkTlsProof);

    // 3. Extract metadata
    let exit_code: [u8; 32] = proof[4..36].try_into().unwrap();
    let vk_root: [u8; 32] = proof[36..68].try_into().unwrap();
    let proof_nonce: [u8; 32] = proof[68..100].try_into().unwrap();
    let raw_proof = &proof[100..356];

    // 4. Verify exit_code == 0
    require!(exit_code == [0u8; 32], crate::errors::KrnError::InvalidZkTlsProof);

    // 5. Verify VK root
    require!(vk_root == SP1_V6_VK_ROOT, crate::errors::KrnError::InvalidZkTlsProof);

    // 6. Decode vkey hash
    let vkey_hash = decode_vkey_hash(KRN_ZKTLS_VKEY_HASH)?;

    // 7. Hash public values (SHA256, zero top 3 bits)
    let committed_values_digest = hash_public_inputs(&proof_data.public_values);

    // 8. Decompress VK from compressed bytes (same as sp1-solana)
    let vk = load_groth16_verifying_key_from_bytes(COMPRESSED_VK)?;

    // 9. Parse and negate proof A (same as sp1-solana)
    let pi_a = negate_g1(&raw_proof[0..64].try_into().unwrap())?;
    let pi_b: [u8; 128] = raw_proof[64..192].try_into().unwrap();
    let pi_c: [u8; 64] = raw_proof[192..256].try_into().unwrap();

    // 10. Construct 5 public inputs
    let public_inputs: [[u8; 32]; 5] = [
        vkey_hash,
        committed_values_digest,
        exit_code,
        vk_root,
        proof_nonce,
    ];

    // 11. Verify
    let vk_struct = Groth16Verifyingkey {
        nr_pubinputs: vk.nr_pubinputs as usize,
        vk_alpha_g1: vk.vk_alpha_g1,
        vk_beta_g2: vk.vk_beta_g2,
        vk_gamme_g2: vk.vk_gamma_g2,
        vk_delta_g2: vk.vk_delta_g2,
        vk_ic: vk.vk_ic.as_slice(),
    };

    let mut verifier = Groth16Verifier::<5>::new(
        &pi_a, &pi_b, &pi_c, &public_inputs, &vk_struct,
    ).map_err(|_| error!(crate::errors::KrnError::InvalidZkTlsProof))?;

    verifier.verify()
        .map_err(|_| error!(crate::errors::KrnError::InvalidZkTlsProof))?;

    msg!("SP1 v6 zkTLS proof verified on-chain. No attestor. Mathematics only.");
    Ok(())
}

// ============================================================
// Below: exact copy of sp1-solana v0.1.0 utils functions
// ============================================================

struct VerificationKey {
    nr_pubinputs: u32,
    vk_alpha_g1: [u8; 64],
    vk_beta_g2: [u8; 128],
    vk_gamma_g2: [u8; 128],
    vk_delta_g2: [u8; 128],
    vk_ic: Vec<[u8; 64]>,
}

fn convert_endianness<const CHUNK_SIZE: usize, const ARRAY_SIZE: usize>(
    bytes: &[u8; ARRAY_SIZE],
) -> [u8; ARRAY_SIZE] {
    let reversed: [_; ARRAY_SIZE] = bytes
        .chunks_exact(CHUNK_SIZE)
        .flat_map(|chunk| chunk.iter().rev().copied())
        .enumerate()
        .fold([0u8; ARRAY_SIZE], |mut acc, (i, v)| {
            acc[i] = v;
            acc
        });
    reversed
}

const GNARK_MASK: u8 = 0b11 << 6;
const GNARK_COMPRESSED_POSITIVE: u8 = 0b10 << 6;
const GNARK_COMPRESSED_NEGATIVE: u8 = 0b11 << 6;
const GNARK_COMPRESSED_INFINITY: u8 = 0b01 << 6;
const ARK_MASK: u8 = 0b11 << 6;
const ARK_COMPRESSED_POSITIVE: u8 = 0b00 << 6;
const ARK_COMPRESSED_NEGATIVE: u8 = 0b10 << 6;
const ARK_COMPRESSED_INFINITY: u8 = 0b01 << 6;

fn gnark_flag_to_ark_flag(msb: u8) -> Result<u8> {
    let gnark_flag = msb & GNARK_MASK;
    let ark_flag = match gnark_flag {
        GNARK_COMPRESSED_POSITIVE => ARK_COMPRESSED_POSITIVE,
        GNARK_COMPRESSED_NEGATIVE => ARK_COMPRESSED_NEGATIVE,
        GNARK_COMPRESSED_INFINITY => ARK_COMPRESSED_INFINITY,
        _ => return Err(error!(crate::errors::KrnError::InvalidZkTlsProof)),
    };
    Ok(msb & !ARK_MASK | ark_flag)
}

fn gnark_compressed_x_to_ark_compressed_x(x: &[u8]) -> Result<Vec<u8>> {
    let mut x_copy = x.to_owned();
    let msb = gnark_flag_to_ark_flag(x_copy[0])?;
    x_copy[0] = msb;
    x_copy.reverse();
    Ok(x_copy)
}

fn decompress_g1(g1_bytes: &[u8; 32]) -> Result<[u8; 64]> {
    let g1_bytes = gnark_compressed_x_to_ark_compressed_x(g1_bytes)?;
    let g1_bytes = convert_endianness::<32, 32>(&g1_bytes.as_slice().try_into().unwrap());
    groth16_solana::decompression::decompress_g1(&g1_bytes)
        .map_err(|_| error!(crate::errors::KrnError::InvalidZkTlsProof))
}

fn decompress_g2(g2_bytes: &[u8; 64]) -> Result<[u8; 128]> {
    let g2_bytes = gnark_compressed_x_to_ark_compressed_x(g2_bytes)?;
    let g2_bytes = convert_endianness::<64, 64>(&g2_bytes.as_slice().try_into().unwrap());
    groth16_solana::decompression::decompress_g2(&g2_bytes)
        .map_err(|_| error!(crate::errors::KrnError::InvalidZkTlsProof))
}

fn uncompressed_bytes_to_g1_point(buf: &[u8]) -> Result<G1Affine> {
    let (x_bytes, y_bytes) = buf.split_at(32);
    let x = Fq::from_be_bytes_mod_order(x_bytes);
    let y = Fq::from_be_bytes_mod_order(y_bytes);
    Ok(G1Affine::new_unchecked(x, y))
}

fn negate_g1(g1_bytes: &[u8; 64]) -> Result<[u8; 64]> {
    let g1 = -uncompressed_bytes_to_g1_point(g1_bytes)?;
    let mut g1_out = [0u8; 64];
    g1.serialize_uncompressed(&mut g1_out[..])
        .map_err(|_| error!(crate::errors::KrnError::InvalidZkTlsProof))?;
    Ok(convert_endianness::<32, 64>(&g1_out))
}

fn load_groth16_verifying_key_from_bytes(buffer: &[u8]) -> Result<VerificationKey> {
    let g1_alpha = decompress_g1(buffer[..32].try_into().unwrap())?;
    let g2_beta = decompress_g2(buffer[64..128].try_into().unwrap())?;
    let g2_gamma = decompress_g2(buffer[128..192].try_into().unwrap())?;
    let g2_delta = decompress_g2(buffer[224..288].try_into().unwrap())?;

    let num_k = u32::from_be_bytes([buffer[288], buffer[289], buffer[290], buffer[291]]);
    let mut k = Vec::new();
    let mut offset = 292;
    for _ in 0..num_k {
        let point = decompress_g1(&buffer[offset..offset + 32].try_into().unwrap())?;
        k.push(point);
        offset += 32;
    }

    let num_of_array_of_public_and_commitment_committed = u32::from_be_bytes([
        buffer[offset], buffer[offset + 1], buffer[offset + 2], buffer[offset + 3],
    ]);

    Ok(VerificationKey {
        vk_alpha_g1: g1_alpha,
        vk_beta_g2: g2_beta,
        vk_gamma_g2: g2_gamma,
        vk_delta_g2: g2_delta,
        vk_ic: k,
        nr_pubinputs: num_of_array_of_public_and_commitment_committed,
    })
}

fn hash_public_inputs(public_inputs: &[u8]) -> [u8; 32] {
    let mut result: [u8; 32] = Sha256::digest(public_inputs).into();
    result[0] &= 0x1F;
    result
}

fn decode_vkey_hash(vkey_hex: &str) -> Result<[u8; 32]> {
    let mut result = [0u8; 32];
    let hex_str = &vkey_hex[2..];
    for i in 0..32 {
        result[i] = u8::from_str_radix(&hex_str[i*2..i*2+2], 16)
            .map_err(|_| error!(crate::errors::KrnError::InvalidZkTlsProof))?;
    }
    Ok(result)
}

