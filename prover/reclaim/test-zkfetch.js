require('dotenv').config();
const { ReclaimClient } = require('@reclaimprotocol/zk-fetch');

async function main() {
  const appId = process.env.RECLAIM_APP_ID;
  const appSecret = process.env.RECLAIM_APP_SECRET;

  console.log('Initializing ReclaimClient...');
  const client = new ReclaimClient(appId, appSecret);

  console.log('Fetching BTC price from CoinGecko with zkProof...');
  try {
    const proof = await client.zkFetch(
      'https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd',
      {
        method: 'GET',
        headers: { accept: 'application/json' },
      },
      {
        responseMatches: [
          {
            type: 'regex',
            value: 'bitcoin.*?usd.*?(?<price>[0-9.]+)',
          },
        ],
        responseRedactions: [
          { regex: 'bitcoin.*?usd.*?(?<price>[0-9.]+)' },
        ],
      }
    );

    console.log('\n=== ZK PROOF GENERATED ===');
    console.log('Proof object keys:', Object.keys(proof));
    console.log('Claim data:', JSON.stringify(proof.claimData, null, 2));
    console.log('\nProof size:', JSON.stringify(proof).length, 'bytes');
    console.log('=== SUCCESS ===');
  } catch (err) {
    console.error('Error:', err.message);
    if (err.stack) console.error(err.stack);
  }
}

main();
