use anyhow::Result;
use sp1_sdk::blocking::{ProverClient, Prover, ProveRequest};
use sp1_sdk::{Elf, SP1Stdin, HashableKey, ProvingKey};
use std::fs;
use zktls_program_core::Request;
use zktls_core::InputBuilder;
use zktls_input_builder::TLSInputBuilder;

const KRN_ZKTLS_ELF: &[u8] = include_bytes!(
    "../../program/target/elf-compilation/riscv64im-succinct-zkvm-elf/release/krn-zktls-guest"
);

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let input_path = args.get(1).map(|s| s.as_str()).unwrap_or("../testdata/input.json");
    let mode = args.get(2).map(|s| s.as_str()).unwrap_or("record-and-mock");
    let threshold: u64 = args.get(3).map(|s| s.parse().expect("Invalid threshold")).unwrap_or(100000);
    let comparison: u8 = args.get(4).map(|s| s.parse().expect("Invalid comparison")).unwrap_or(0);

    println!("KRN zkTLS Prover (SP1 v6)");
    println!("Mode: {}", mode);
    println!("Input: {}", input_path);
    println!("Threshold: {}, Comparison: {} (0=gt, 1=lt, 2=eq)", threshold, comparison);

    let request_json = fs::read_to_string(input_path)?;
    let request: Request = serde_json::from_str(&request_json)?;

    println!("Recording TLS session to {}...", request.request_info.remote_addr);
    let mut builder = TLSInputBuilder::new()?;
    let guest_input = builder.build_input(request).await?;

    println!("TLS session recorded successfully");
    println!("Response size: {} bytes", guest_input.response.response.len());

    let output_dir = "../output";
    fs::create_dir_all(output_dir)?;
    let guest_input_json = serde_json::to_string_pretty(&guest_input)?;
    fs::write(format!("{}/guest_input.json", output_dir), &guest_input_json)?;
    println!("Saved guest_input.json");

    if mode == "record-only" {
        return Ok(());
    }

    let mut input_bytes = Vec::new();
    ciborium::into_writer(&guest_input, &mut input_bytes)?;

    match mode {
        "record-and-mock" | "mock" => std::env::set_var("SP1_PROVER", "mock"),
        "cpu" => std::env::set_var("SP1_PROVER", "cpu"),
        "cuda" => std::env::set_var("SP1_PROVER", "cuda"),
        "network" => std::env::set_var("SP1_PROVER", "network"),
        _ => std::env::set_var("SP1_PROVER", "mock"),
    }

    let (proof_output, elapsed, vkey_hash) = tokio::task::spawn_blocking(move || {
        let client = ProverClient::from_env();
        let pk = client.setup(Elf::Static(KRN_ZKTLS_ELF)).expect("Setup failed");

        let vk = pk.verifying_key();
        let vkey_hash = vk.bytes32();
        println!("VKey hash: {}", vkey_hash);
        println!("Generating proof...");

        let mut stdin = SP1Stdin::new();
        stdin.write_vec(input_bytes);
        stdin.write(&threshold);
        stdin.write(&comparison);

        let start = std::time::Instant::now();
        let proof = client
            .prove(&pk, stdin)
            .groth16()
            .run()
            .expect("Proof generation failed");
        let elapsed = start.elapsed();
        (proof, elapsed, vkey_hash)
    }).await?;

    println!("Proof generated in {:?}", elapsed);

    let public_values = proof_output.public_values.to_vec();
    let proof_bytes = proof_output.bytes();

    println!("Public values: {} bytes", public_values.len());
    println!("Proof: {} bytes", proof_bytes.len());
    println!("VKey hash: {}", vkey_hash);

    fs::write(format!("{}/proof.bin", output_dir), &proof_bytes)?;
    fs::write(format!("{}/public_values.bin", output_dir), &public_values)?;
    fs::write(format!("{}/vkey_hash.txt", output_dir), &vkey_hash)?;
    proof_output.save(format!("{}/full_proof.bin", output_dir))?;

    println!("Saved to {}/", output_dir);

    Ok(())
}
