#![no_main]
sp1_zkvm::entrypoint!(main);


/// KRN zkTLS Guest Program
///
/// Runs inside SP1 zkVM to trustlessly verify TLS data from API sources.
/// No attestor needed. TLS certificate chain verified against root CAs.
///
/// Public outputs committed to proof:
///   - server_name_hash: keccak256(server_name) for domain matching
///   - response_data: extracted HTTP response bytes
///   - request_hash: unique identifier for this request
///   - dapp_address: address that initiated the request
fn main() {
    // Read TLS session input (recorded off-chain)
    let input = sp1_zkvm::io::read_vec();

    // Replay and verify TLS connection inside zkVM
    // This verifies certificate chain against webpki root CAs
    // If cert is invalid or data tampered, this panics and proof fails
    let output = zktls_replayable_tls::entry(&input);

    // Commit verified output as public values
    // On-chain verifier will parse these to extract domain + outcome
    sp1_zkvm::io::commit_slice(&output);
}
