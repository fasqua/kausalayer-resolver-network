const { buildPoseidon } = require("circomlibjs");

async function main() {
  const poseidon = await buildPoseidon();
  const F = poseidon.F;

  const market_id = 12345n;
  const resolved_outcome = 1n;
  const secret_nonce = 98765432101234567890n;
  const amount = 500000000n;
  const original_pubkey = 11111111111111111111n;

  // commitment = Poseidon(market_id, resolved_outcome, amount, secret_nonce, original_pubkey)
  const commitment = poseidon([market_id, resolved_outcome, amount, secret_nonce, original_pubkey]);
  console.log("commitment:", F.toString(commitment));

  // nullifier = Poseidon(market_id, secret_nonce, original_pubkey)
  const nullifier = poseidon([market_id, secret_nonce, original_pubkey]);
  console.log("nullifier:", F.toString(nullifier));

  // Simplified Merkle: single level with sibling=0, direction=0
  // root = Poseidon(commitment, 0)
  const sibling = 0n;
  const root = poseidon([commitment, sibling]);
  console.log("commitment_root:", F.toString(root));

  const input = {
    market_id: market_id.toString(),
    resolved_outcome: resolved_outcome.toString(),
    secret_nonce: secret_nonce.toString(),
    amount: amount.toString(),
    original_pubkey: original_pubkey.toString(),
    sibling: sibling.toString(),
    direction: "0",
    nullifier: F.toString(nullifier),
    commitment_root: F.toString(root),
  };

  require("fs").writeFileSync("build/input.json", JSON.stringify(input, null, 2));
  console.log("\nInput written to build/input.json");
  console.log(JSON.stringify(input, null, 2));
}

main().catch(console.error);
