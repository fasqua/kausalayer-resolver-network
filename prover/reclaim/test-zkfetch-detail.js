require('dotenv').config();
const { ReclaimClient } = require('@reclaimprotocol/zk-fetch');
const fs = require('fs');

async function main() {
  const client = new ReclaimClient(
    process.env.RECLAIM_APP_ID,
    process.env.RECLAIM_APP_SECRET
  );

  console.log('Generating zkFetch proof from CoinGecko...');
  const proof = await client.zkFetch(
    'https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd',
    {
      method: 'GET',
      headers: { accept: 'application/json' },
    },
    {
      responseMatches: [
        { type: 'regex', value: 'bitcoin.*?usd.*?(?<price>[0-9.]+)' },
      ],
      responseRedactions: [
        { regex: 'bitcoin.*?usd.*?(?<price>[0-9.]+)' },
      ],
    }
  );

  // Save full proof to file
  fs.writeFileSync('proof-sample.json', JSON.stringify(proof, null, 2));
  console.log('Full proof saved to proof-sample.json\n');

  // Analyze structure
  console.log('=== PROOF STRUCTURE ANALYSIS ===\n');

  console.log('1. IDENTIFIER:', proof.identifier);
  console.log('   Type:', typeof proof.identifier);
  console.log('   Length:', proof.identifier.length, 'chars\n');

  console.log('2. SIGNATURES:', JSON.stringify(proof.signatures, null, 2));
  console.log('   Count:', proof.signatures.length, '\n');

  console.log('3. WITNESSES:', JSON.stringify(proof.witnesses, null, 2));
  console.log('   Count:', proof.witnesses.length, '\n');

  console.log('4. EXTRACTED PARAMS:', JSON.stringify(proof.extractedParameterValues, null, 2), '\n');

  // Parse claimData
  const claim = proof.claimData;
  console.log('5. CLAIM DATA:');
  console.log('   Provider:', claim.provider);
  console.log('   Owner:', claim.owner);
  console.log('   Timestamp:', claim.timestampS, '(' + new Date(claim.timestampS * 1000).toISOString() + ')');
  console.log('   Epoch:', claim.epoch);
  console.log('   Identifier:', claim.identifier);

  const context = JSON.parse(claim.context);
  console.log('   Extracted price:', context.extractedParameters.price);
  console.log('   Provider hash:', context.providerHash);

  const params = JSON.parse(claim.parameters);
  console.log('   URL:', params.url);
  console.log('   Method:', params.method);

  console.log('\n=== KEY FIELDS FOR ON-CHAIN VERIFICATION ===');
  console.log('identifier (bytes32):', proof.identifier);
  console.log('epoch (u8):', claim.epoch);
  console.log('timestampS (i64):', claim.timestampS);
  console.log('owner (address):', claim.owner);
  console.log('signatures:', proof.signatures.length, 'signature(s)');
  console.log('witnesses:', proof.witnesses.length, 'witness(es)');
}

main().catch(console.error);
