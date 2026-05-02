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
        jsonPathHash: sha256("$.bitcoin.usd"),
      },
      {
        domainHash: sha256("api.coingecko.com"),
        pathHash: sha256("/api/v3/simple/price"),
        jsonPathHash: sha256("$.bitcoin.usd"),
      },
      {
        domainHash: sha256("api.coingecko.com"),
        pathHash: sha256("/api/v3/simple/price"),
        jsonPathHash: sha256("$.bitcoin.usd"),
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
    const proofHex = "4388a21c0000000000000000000000000000000000000000000000000000000000000000002f850ee998974d6cc00e50cd0814b098c05bfade466d28573240d057f25352000000000000000000000000000000000000000000000000000000000000000014089cabd5b2b8858aed9f0f2c172e010be278b34e94f4ab3cd5d84c294c25b711472284be966bcd8b6c687618fe39c230486aba35bedc18e0015dc0ed64109f0524ad655e0e0c92d791e33897e1aa80d64e708d4bff3e299455614b7bc283dd1a31dc207028c2271f9b9f16e9c8ed0d942d4ea2f9b6ce2ee15ff3ada181cb5d2d97bba1658391285218e4d3ab15179d98325a14d48d7e00340f67c5d7d113980c15876f479fa384acc49d569e5208e294175c070bb7c315464173d4d15b14970052a1656aecd310afadc59eedee5da95f6715b26edfabae5c047fbd1cc759091f2a8af6ca1b199acb99383bc5d0ed3df6d60ed0890d8a10aecdc18b8a2eeddb";
    const pvHex = "0161962e9472e133f50008e6c9b0a088f327a79164870c4a0b3b01d215d4726c2eb55c46630ce8efb43c31878d1955c89c0bafea9f95222290dd7278aa3ddd389cc1e1d165cc4bafe5e19cb336d24b30c013e7bdb2e93659d6086672be7191a02262a7e032ceb43fc9000000000000000100000000724b524e000000000000000186a00000000000013252116170692e636f696e6765636b6f2e636f6d";

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
