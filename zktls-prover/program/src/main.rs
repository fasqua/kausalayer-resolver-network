#![no_main]
sp1_zkvm::entrypoint!(main);

/// KRN zkTLS Guest Program v3
///
/// Runs inside SP1 zkVM to trustlessly verify TLS data from API sources.
/// No attestor needed. TLS certificate chain verified against root CAs.
///
/// Flow:
///   1. Replay and verify TLS connection inside zkVM
///   2. Commit original zkTLS output (backward compatible)
///   3. Read market parameters (threshold, comparison, json_path)
///   4. Parse HTTP response body, extract value at specified JSON path
///   5. Determine outcome (YES/NO) based on threshold comparison
///   6. Commit structured KRN output
///
/// Public outputs:
///   - Original zkTLS output (variable length)
///   - original_output_length: 4 bytes (u32 big-endian)
///   - KRN marker: 0x4B524E (3 bytes, "KRN")
///   - version: 1 byte (0x02 for v3 with json_path)
///   - outcome: 1 byte (0=NO, 1=YES)
///   - comparison: 1 byte (0=gt, 1=lt, 2=eq)
///   - threshold: 8 bytes (u64 big-endian)
///   - extracted_value: 8 bytes (u64 big-endian)
///   - server_name_length: 1 byte
///   - server_name: variable bytes
///   - json_path_length: 1 byte
///   - json_path: variable bytes (raw path for on-chain validation)
fn main() {
    // Read TLS session input (recorded off-chain)
    let input = sp1_zkvm::io::read_vec();

    // Read market parameters
    let threshold: u64 = sp1_zkvm::io::read();
    let comparison: u8 = sp1_zkvm::io::read();
    let json_path: Vec<u8> = sp1_zkvm::io::read_vec();

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

    // Extract numeric value(s) at specified JSON path
    // Path format: "bitcoin.usd" or "competitors.0.score" (dot-separated keys)
    // For comparison mode 3 (first_greater): path contains two paths separated by "|"
    // e.g. "events.0.competitions.0.competitors.0.score|events.0.competitions.0.competitors.1.score"
    let (extracted_value, extracted_value_b) = if comparison == 3 {
        // Split path by '|' delimiter
        let split_pos = json_path.iter().position(|&b| b == b'|')
            .expect("Comparison mode 3 requires two paths separated by |");
        let path_a = &json_path[..split_pos];
        let path_b = &json_path[split_pos + 1..];
        let val_a = extract_value_at_path(body, path_a);
        let val_b = extract_value_at_path(body, path_b);
        (val_a, val_b)
    } else if json_path.is_empty() {
        // Backward compatible: no path = extract first number (v2 behavior)
        (extract_json_number(body), 0u64)
    } else {
        (extract_value_at_path(body, &json_path), 0u64)
    };

    // Determine outcome based on threshold comparison
    let outcome: u8 = match comparison {
        0 => if extracted_value > threshold { 1 } else { 0 }, // greater_than
        1 => if extracted_value < threshold { 1 } else { 0 }, // less_than
        2 => if extracted_value == threshold { 1 } else { 0 }, // equal
        3 => if extracted_value > extracted_value_b { 1 } else { 0 }, // first_greater (head-to-head)
        _ => panic!("Invalid comparison type"),
    };

    // Commit KRN structured output
    // Length prefix for deterministic parsing (no marker scanning)
    sp1_zkvm::io::commit_slice(&(output.len() as u32).to_be_bytes());
    sp1_zkvm::io::commit_slice(&[0x4B, 0x52, 0x4E]); // "KRN" sanity check marker
    sp1_zkvm::io::commit_slice(&[0x02]); // version 2 (v3 guest with json_path)
    sp1_zkvm::io::commit_slice(&[outcome]);
    sp1_zkvm::io::commit_slice(&[comparison]);
    sp1_zkvm::io::commit_slice(&threshold.to_be_bytes());
    sp1_zkvm::io::commit_slice(&extracted_value.to_be_bytes());
    sp1_zkvm::io::commit_slice(&[server_name.len() as u8]);
    sp1_zkvm::io::commit_slice(server_name);
    sp1_zkvm::io::commit_slice(&[json_path.len() as u8]);
    sp1_zkvm::io::commit_slice(&json_path);
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

/// Extract numeric value at a dot-separated JSON path.
/// Supports paths like: "bitcoin.usd", "competitors.0.score", "data.amount"
/// Array indices are specified as numeric keys: "competitors.0.score"
/// Returns the integer part of the value found at the path.
fn extract_value_at_path(json: &[u8], path: &[u8]) -> u64 {
    let mut current = json;

    // Split path by '.' and navigate each segment
    let mut seg_start = 0;
    let mut seg_end = 0;
    let path_len = path.len();

    loop {
        // Find next segment boundary
        while seg_end < path_len && path[seg_end] != b'.' {
            seg_end += 1;
        }

        let segment = &path[seg_start..seg_end];

        // Check if segment is array index (all digits)
        let is_index = segment.iter().all(|b| b.is_ascii_digit());

        if is_index {
            let index = parse_usize_bytes(segment);
            current = navigate_to_array_element(current, index);
        } else {
            current = navigate_to_key(current, segment);
        }

        // Move past the dot
        if seg_end >= path_len {
            break;
        }
        seg_end += 1;
        seg_start = seg_end;
    }

    // Extract number from current position
    extract_json_number(current)
}

/// Navigate JSON bytes to find the value after a specific key.
/// Given {"foo":{"bar":42}}, navigate_to_key(json, "foo") returns {"bar":42}
fn navigate_to_key<'a>(json: &'a [u8], key: &[u8]) -> &'a [u8] {
    let len = json.len();
    let key_len = key.len();
    let mut i = 0;

    while i < len {
        // Look for quote that starts a key
        if json[i] == b'"' {
            let key_start = i + 1;
            // Check if this key matches
            if key_start + key_len < len
                && &json[key_start..key_start + key_len] == key
                && json[key_start + key_len] == b'"'
            {
                // Found matching key, skip to colon and value
                let mut j = key_start + key_len + 1;
                // Skip whitespace and colon
                while j < len && (json[j] == b':' || json[j] == b' ' || json[j] == b'\t' || json[j] == b'\n' || json[j] == b'\r') {
                    j += 1;
                }
                return &json[j..];
            }
        }
        i += 1;
    }

    panic!("JSON key not found in response");
}

/// Navigate to the nth element in a JSON array.
/// Given [{"a":1},{"a":2}], navigate_to_array_element(json, 1) returns {"a":2}
fn navigate_to_array_element<'a>(json: &'a [u8], index: usize) -> &'a [u8] {
    let len = json.len();
    let mut i = 0;

    // Find opening bracket
    while i < len && json[i] != b'[' {
        i += 1;
    }
    i += 1; // skip '['

    // Skip to the nth element by counting commas at depth 0
    let mut current_index = 0;
    let mut depth = 0;

    while i < len && current_index < index {
        match json[i] {
            b'{' | b'[' => depth += 1,
            b'}' | b']' => depth -= 1,
            b',' if depth == 0 => current_index += 1,
            _ => {}
        }
        i += 1;
    }

    // Skip whitespace
    while i < len && (json[i] == b' ' || json[i] == b'\t' || json[i] == b'\n' || json[i] == b'\r') {
        i += 1;
    }

    &json[i..]
}

/// Parse a usize from ASCII digit bytes. "123" -> 123
fn parse_usize_bytes(bytes: &[u8]) -> usize {
    let mut result: usize = 0;
    for &b in bytes {
        if b.is_ascii_digit() {
            result = result * 10 + (b - b'0') as usize;
        }
    }
    result
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
