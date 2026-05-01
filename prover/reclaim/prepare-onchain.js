const { ethers } = require('ethers');
const fs = require('fs');

// Load proof
const proof = JSON.parse(fs.readFileSync('proof-sample.json', 'utf8'));

console.log('=== PREPARE RECLAIM PROOF FOR ON-CHAIN SUBMISSION ===\n');

// 1. identifier: 32 bytes
const identifier = Array.from(ethers.getBytes(proof.claimData.identifier));
console.log('identifier (32 bytes):', identifier.length);

// 2. owner: 20 bytes
const owner = Array.from(ethers.getBytes(proof.claimData.owner));
console.log('owner (20 bytes):', owner.length);

// 3. timestamp_s: u64
const timestampS = proof.claimData.timestampS;
console.log('timestamp_s:', timestampS);

// 4. epoch: u8
const epoch = proof.claimData.epoch;
console.log('epoch:', epoch);

// 5. signature: 64 bytes (r + s) and recovery_id: u8
const sigBytes = ethers.getBytes(proof.signatures[0]);
const signature = Array.from(sigBytes.slice(0, 64));
const recoveryId = sigBytes[64] - 27; // v (27 or 28) -> recovery_id (0 or 1)
console.log('signature (64 bytes):', signature.length);
console.log('recovery_id:', recoveryId);

// Build the struct as it would be passed to Anchor
const reclaimProof = {
  identifier,
  owner,
  timestampS,
  epoch,
  signature,
  recoveryId,
};

console.log('\n=== ANCHOR-READY OBJECT ===');
console.log(JSON.stringify(reclaimProof, null, 2));

// Save for use in tests
fs.writeFileSync('proof-onchain.json', JSON.stringify(reclaimProof, null, 2));
console.log('\nSaved to proof-onchain.json');
