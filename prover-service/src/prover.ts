import { execSync } from "child_process";
import path from "path";
import fs from "fs";

/**
 * Proof result from SP1 Succinct Prover Network.
 */
export interface ProofResult {
  proof: Buffer;
  publicValues: Buffer;
}

/**
 * Generate a zkTLS proof via the SP1 Succinct Prover Network.
 *
 * Wraps the existing Rust prover binary (zktls-prover/script) which:
 * 1. Records TLS session to the target API
 * 2. Sends to Succinct Network for Groth16 proving
 * 3. Outputs proof.bin and public_values.bin
 *
 * @param domain - Target API domain (e.g. "api.coingecko.com")
 * @param apiPath - API path (e.g. "/api/v3/simple/price?ids=bitcoin&vs_currencies=usd")
 * @param threshold - Market threshold value
 * @param comparison - Comparison type (0=gt, 1=lt, 2=eq, 3=first_greater)
 * @param jsonPath - JSON path for value extraction
 */
export async function generateProof(
  domain: string,
  apiPath: string,
  threshold: bigint,
  comparison: number,
  jsonPath: string,
): Promise<ProofResult> {
  const repoRoot = path.resolve(__dirname, "../..");
  const scriptDir = path.resolve(repoRoot, "zktls-prover/script");
  const outputDir = path.resolve(repoRoot, "zktls-prover/output");

  // Build input JSON for the prover
  const requestHex = Buffer.from(
    "GET " + apiPath + " HTTP/1.1\r\n" +
    "Host: " + domain + "\r\n" +
    "Accept: */*\r\n" +
    "User-Agent: KausaLayer-KRN/1.0\r\n" +
    "Connection: close\r\n" +
    "\r\n"
  ).toString("hex");

  const inputJson = {
    version: 1,
    request_info: {
      request: "0x" + requestHex,
      remote_addr: domain + ":443",
      server_name: domain,
    },
    response_template: [],
    target: {
      client: "0x95222290dd7278aa3ddd389cc1e1d165cc4bafe5",
      prover_id: "0xe19cb336d24b30c013e7bdb2e93659d6086672be7191a02262a7e032ceb43fc9",
      submit_network_id: 1,
    },
    origin: {
      type: "secp256k1",
      signature: "0x61600537178396fc1cb1abf2d880d6f0805d8969f672c4181857436ae5d0225875ffd4a212ced58dabe760b7e248a3f9ab1c9acf32bce1983e05c1ba9e3e228700",
      nonce: 0,
    },
  };

  // Write input file
  const inputPath = path.resolve(outputDir, "prover-service-input.json");
  fs.mkdirSync(outputDir, { recursive: true });
  fs.writeFileSync(inputPath, JSON.stringify(inputJson, null, 2));

  console.log("[prover] Input written to:", inputPath);
  console.log("[prover] Generating proof via Succinct Network...");
  console.log("[prover] Domain:", domain);
  console.log("[prover] Threshold:", threshold.toString(), "Comparison:", comparison);

  try {
    // Call Rust prover binary in network mode
    const cmd = [
      "cargo run --release --",
      inputPath,
      "network",
      threshold.toString(),
      comparison.toString(),
      jsonPath ? '"' + jsonPath + '"' : '""'
    ].join(" ");

    const output = execSync(cmd, {
      cwd: scriptDir,
      timeout: 300000, // 5 minute timeout
      stdio: "pipe",
      env: { ...process.env },
    });

    console.log("[prover] Prover output:", output.toString().slice(-200));

    // Read proof artifacts
    const proofPath = path.resolve(outputDir, "proof.bin");
    const pvPath = path.resolve(outputDir, "public_values.bin");

    if (!fs.existsSync(proofPath) || !fs.existsSync(pvPath)) {
      throw new Error("Proof artifacts not found after proving");
    }

    const proof = fs.readFileSync(proofPath);
    const publicValues = fs.readFileSync(pvPath);

    console.log("[prover] Proof generated: proof=" + proof.length + " bytes, pv=" + publicValues.length + " bytes");

    return { proof, publicValues };
  } catch (err: any) {
    console.error("[prover] Proof generation failed:", err.message);
    throw err;
  }
}
