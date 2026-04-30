const { ethers } = require('ethers');
const canonicalize = require('canonicalize');
const fs = require('fs');

const proof = JSON.parse(fs.readFileSync('proof-sample.json', 'utf8'));

console.log('=== OFF-CHAIN VERIFICATION TEST ===\n');

function getIdentifier(claimData) {
  let canonicalContext = claimData.context || '';
  if (canonicalContext.length > 0) {
    const ctx = JSON.parse(canonicalContext);
    canonicalContext = canonicalize(ctx);
  }
  const str = `${claimData.provider}\n${claimData.parameters}\n${canonicalContext}`;
  return ethers.keccak256(ethers.toUtf8Bytes(str)).toLowerCase();
}

function createSignData(claimData) {
  const identifier = getIdentifier(claimData);
  return [
    identifier,
    claimData.owner.toLowerCase(),
    claimData.timestampS.toString(),
    claimData.epoch.toString(),
  ].join('\n');
}

const identifier = getIdentifier(proof.claimData);
const signedData = createSignData(proof.claimData);

console.log('1. Computed identifier:', identifier);
console.log('   Proof identifier:  ', proof.claimData.identifier);
console.log('   Match:', identifier === proof.claimData.identifier);

const signature = proof.signatures[0];
const recoveredAddress = ethers.verifyMessage(signedData, signature);

console.log('\n2. Recovered signer:', recoveredAddress.toLowerCase());
console.log('   Expected attestor:', proof.witnesses[0].id.toLowerCase());
console.log('   Match:', recoveredAddress.toLowerCase() === proof.witnesses[0].id.toLowerCase());

const sigBytes = ethers.getBytes(signature);
const v = sigBytes[64];
const recoveryId = v - 27;

console.log('\n3. Signature details:');
console.log('   v:', v, '(recovery_id:', recoveryId, ')');
console.log('   Signed data length:', ethers.toUtf8Bytes(signedData).length);

if (identifier === proof.claimData.identifier &&
    recoveredAddress.toLowerCase() === proof.witnesses[0].id.toLowerCase()) {
  console.log('\n=== ALL CHECKS PASSED ===');
} else {
  console.log('\n=== VERIFICATION FAILED ===');
}
