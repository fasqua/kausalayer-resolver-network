#![no_main]
sp1_zkvm::entrypoint!(main);

/// KRN zkTLS Guest Program v2
///
/// Runs inside SP1 zkVM to trustlessly verify TLS data from API sources.
/// No attestor needed. TLS certificate chain verified against root CAs.
///
/// Flow:
///   1. Replay and verify TLS connection inside zkVM
///   2. Commit original zkTLS output (backward compatible)
///   3. Read market parameters (threshold, comparison)
///   4. Parse HTTP response body, extract price value
///   5. Determine outcome (YES/NO) based on threshold comparison
///   6. Commit structured KRN output: [outcome, threshold, extracted_value, server_name_hash]
///
/// Public outputs:
///   - Original zkTLS output (variable length)
///   - original_output_length: 4 bytes (u32 big-endian)
///   - KRN marker: 0x4B524E (3 bytes, "KRN")
///   - outcome: 1 byte (0=NO, 1=YES)
///   - comparison: 1 byte (0=gt, 1=lt, 2=eq)
///   - threshold: 8 bytes (u64 big-endian)
///   - extracted_value: 8 bytes (u64 big-endian, price in cents or smallest unit)
///   - server_name_length: 1 byte
///   - server_name: variable bytes (raw server name for on-chain domain validation)
fn main() {
    // Read TLS session input (recorded off-chain)
    let input = sp1_zkvm::io::read_vec();

    // Read market parameters
    let threshold: u64 = sp1_zkvm::io::read();
    let comparison: u8 = sp1_zkvm::io::read();

    // Replay and verify TLS connection inside zkVM
    // This verifies certificate chain against webpki root CAs
    // If cert is invalid or data tampered, this panics and proof fails
    let output = zktls_replayable_tls::entry(&input);

    // Commit original zkTLS output (preserves compatibility)
    sp1_zkvm::io::commit_slice(&output);

    // Parse the TLS input to extract server_name and response body
    let guest_input: zktls_program_core::GuestInput =
        ciborium::from_reader(input.as_slice()).expect("Failed to parse guest input");

    let server_name = guest_input.request.request_info.server_name.as_bytes();
    let response_bytes = &guest_input.response.response;

    // Extract HTTP response body from raw response
    // Response format: "HTTP/1.1 200 OK\r\n...headers...\r\n\r\n{json_body}"
    let body = extract_http_body(response_bytes);

    // Extract numeric value from JSON response
    // Supports formats like: {"bitcoin":{"usd":78383}} or {"data":{"amount":"107234.50"}}
    let extracted_value = extract_json_number(body);

    // Determine outcome based on threshold comparison
    let outcome: u8 = match comparison {
        0 => if extracted_value > threshold { 1 } else { 0 }, // greater_than
        1 => if extracted_value < threshold { 1 } else { 0 }, // less_than
        2 => if extracted_value == threshold { 1 } else { 0 }, // equal
        _ => panic!("Invalid comparison type"),
    };

    // Commit KRN structured output
    // Length prefix for deterministic parsing (no marker scanning)
    sp1_zkvm::io::commit_slice(&(output.len() as u32).to_be_bytes());
    sp1_zkvm::io::commit_slice(&[0x4B, 0x52, 0x4E]); // "KRN" sanity check marker
    sp1_zkvm::io::commit_slice(&[outcome]);
    sp1_zkvm::io::commit_slice(&[comparison]);
    sp1_zkvm::io::commit_slice(&threshold.to_be_bytes());
    sp1_zkvm::io::commit_slice(&extracted_value.to_be_bytes());
    sp1_zkvm::io::commit_slice(&[server_name.len() as u8]);
    sp1_zkvm::io::commit_slice(server_name);
}

/// Extract HTTP body from raw response bytes.
/// Finds "\r\n\r\n" separator between headers and body.
fn extract_http_body(response: &[u8]) -> &[u8] {
    let separator = b"\r\n\r\n";
    for i in 0..response.len().saturating_sub(3) {
        if &response[i..i + 4] == separator {
            return &response[i + 4..];
        }
    }
    // If no separator found, assume entire response is body
    response
}

/// Extract the first numeric value from JSON bytes.
/// Handles both integer (78383) and string decimal ("107234.50") formats.
/// Returns value as u64 (integer part only, no decimals).
fn extract_json_number(json: &[u8]) -> u64 {
    let mut i = 0;
    let len = json.len();

    while i < len {
        // Skip until we find a digit or a quote followed by digits
        if json[i] == b'"' {
            // Check if quoted value is a number: "107234.50"
            let start = i + 1;
            if start < len && (json[start].is_ascii_digit() || json[start] == b'-') {
                let mut end = start;
                while end < len && json[end] != b'"' {
                    end += 1;
                }
                return parse_number_bytes(&json[start..end]);
            }
        } else if json[i] == b':' {
            // After colon, skip whitespace, look for number
            let mut j = i + 1;
            while j < len && (json[j] == b' ' || json[j] == b'\t') {
                j += 1;
            }
            if j < len && (json[j].is_ascii_digit() || json[j] == b'-') {
                let start = j;
                let mut end = start;
                while end < len && (json[end].is_ascii_digit() || json[end] == b'.' || json[end] == b'-') {
                    end += 1;
                }
                return parse_number_bytes(&json[start..end]);
            }
        }
        i += 1;
    }

    panic!("No numeric value found in response");
}

/// Parse number from byte slice. Handles "107234.50" -> 107234, "78383" -> 78383
fn parse_number_bytes(bytes: &[u8]) -> u64 {
    let mut result: u64 = 0;
    let mut negative = false;

    for &b in bytes {
        if b == b'-' {
            negative = true;
        } else if b == b'.' {
            break; // Stop at decimal point, take integer part only
        } else if b.is_ascii_digit() {
            result = result * 10 + (b - b'0') as u64;
        }
    }

    if negative { 0 } else { result }
}
