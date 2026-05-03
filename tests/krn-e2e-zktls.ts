import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Krn } from "../target/types/krn";
import { assert } from "chai";
import { PublicKey, SystemProgram, ComputeBudgetProgram } from "@solana/web3.js";
import { createHash } from "crypto";

function sha256(data: string): number[] {
  return Array.from(createHash("sha256").update(data).digest());
}

describe("KRN E2E: CoinGecko zkTLS Proof On-Chain", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.krn as Program<Krn>;
  const creator = provider.wallet;

  it("Submits real CoinGecko SP1 zkTLS proof via submit_proof", async () => {
    // Market: "BTC > 100k" with CoinGecko as source
    const marketId = sha256("e2e-coingecko-btc-100k-" + Date.now().toString());

    // Source config: domain_hash = sha256("api.coingecko.com")
    const sourceConfigs = [
      {
        domainHash: sha256("api.coingecko.com"),
        pathHash: sha256("/api/v3/simple/price"),
        jsonPathHash: sha256(""),
      },
      {
        domainHash: sha256("api.coingecko.com"),
        pathHash: sha256("/api/v3/simple/price"),
        jsonPathHash: sha256(""),
      },
      {
        domainHash: sha256("api.coingecko.com"),
        pathHash: sha256("/api/v3/simple/price"),
        jsonPathHash: sha256(""),
      },
    ];

    const now = Math.floor(Date.now() / 1000);
    const threshold = new anchor.BN(100000); // BTC > 100k
    const comparison = 0; // greater_than

    const [marketPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("market"), Buffer.from(marketId)],
      program.programId
    );
    const [poolPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), Buffer.from(marketId)],
      program.programId
    );

    // Step 1: Init market
    await program.methods
      .initMarket(marketId, new anchor.BN(now + 3), new anchor.BN(now + 3600), sourceConfigs, 3, threshold, comparison)
      .accounts({ market: marketPda, creator: creator.publicKey, systemProgram: SystemProgram.programId })
      .rpc();
    console.log("1. Market initialized: BTC > 100k (CoinGecko)");

    // Step 2: Wait and close market
    await new Promise((r) => setTimeout(r, 4000));
    await program.methods
      .closeMarket()
      .accounts({ market: marketPda, caller: creator.publicKey })
      .rpc();
    console.log("2. Market closed");

    // Step 3: Submit real CoinGecko SP1 zkTLS proof
    const proofHex = "4388a21c0000000000000000000000000000000000000000000000000000000000000000002f850ee998974d6cc00e50cd0814b098c05bfade466d28573240d057f253520000000000000000000000000000000000000000000000000000000000000000131a8d572d9a88be048d3617499f288def3dc238d940790204414a7781f9813b051a03f93361aaa0fc0ff0ed2750c23e91f4b974cd42631f655a131f969219de2ab4fffd4faeadfc6d40d5848c6b45376c2fcd186d73a512d9bcd0a4bf07e71721e7f4208188274031f84767c2c8e1a6f0854148b07c573a6c87cde5d7989acc268b3a711d16f437e47671643055604d592cf8c8ada97f5550f7706110cbc7ee083139f16ba1490d86917024c70617ef4003db99763328f1f2a72d0d2fd91ccb0eb5427a065691f5ff283d678302eb708f0913f3fa9974dd75179508047d19d32596a6b1483998fa73fd80eab2c5668caebb7e92b869fceab539c55c2c10f211";
    const pvHex = "0161962e9472e133f50008e6c9b0a088f327a79164870c4a0b3b01d215d4726c2eb55c46630ce8efb43c31878d1955c89c0bafea9f95222290dd7278aa3ddd389cc1e1d165cc4bafe5e19cb336d24b30c013e7bdb2e93659d6086672be7191a02262a7e032ceb43fc9000000000000000100000000724b524e02000000000000000186a0000000000001335b116170692e636f696e6765636b6f2e636f6d00";

    const proof = Buffer.from(proofHex, "hex");
    const publicValues = Buffer.from(pvHex, "hex");

    const [proofPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("proof"), Buffer.from(marketId), Buffer.from([0])],
      program.programId
    );

    const submitTx = await program.methods
      .submitProof(marketId, 0, {
        proof: proof,
        publicValues: publicValues,
      })
      .accounts({
        market: marketPda,
        proofSubmission: proofPda,
        submitter: creator.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 }),
      ])
      .rpc();

    console.log("3. SP1 zkTLS proof submitted via submit_proof");
    console.log("   TX:", submitTx);
    console.log("   Explorer: https://explorer.solana.com/tx/" + submitTx + "?cluster=devnet");

    // Verify market state
    const market = await program.account.marketAccount.fetch(marketPda);
    assert.equal(market.sourceCount, 1);
    assert.deepInclude(market.state, { resolving: {} });
    console.log("   Market state: Resolving, sourceCount: 1");

    // Verify proof submission
    const proofSubmission = await program.account.proofSubmission.fetch(proofPda);
    assert.equal(proofSubmission.claimedOutcome, 0); // NO (BTC 78418 < 100k)
    assert.equal(proofSubmission.verified, true);
    console.log("   Proof verified: outcome=NO (BTC ~78418 < 100000)");

    console.log("\n=== E2E COINGECKO zkTLS PROOF VERIFIED ON SOLANA DEVNET ===");
    console.log("Real BTC price from CoinGecko API, verified inside SP1 zkVM,");
    console.log("Groth16 proof submitted and verified on-chain via submit_proof.");
    console.log("No attestor. No oracle. Mathematics only.");
  });
});
