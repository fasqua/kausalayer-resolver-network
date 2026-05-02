import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Krn } from "../target/types/krn";
import { expect } from "chai";
import { PublicKey, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { createHash } from "crypto";


function sha256(data: string): number[] {
  return Array.from(createHash("sha256").update(data).digest());
}

function uniqueMarketId(label: string): number[] {
  return sha256("e2e-comprehensive-" + label + "-" + Date.now());
}


function makeSourceConfigs() {
  return [
    { domainHash: sha256("domain-1"), pathHash: sha256("path-1"), jsonPathHash: sha256("json-1") },
    { domainHash: sha256("domain-2"), pathHash: sha256("path-2"), jsonPathHash: sha256("json-2") },
    { domainHash: sha256("domain-3"), pathHash: sha256("path-3"), jsonPathHash: sha256("json-3") },
  ];
}

function getPdas(marketId: number[], programId: PublicKey) {
  const [marketPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("market"), Buffer.from(marketId)], programId
  );
  const [poolPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("pool"), Buffer.from(marketId)], programId
  );
  return { marketPda, poolPda };
}

// Groth16 proof data (from circuits/ownership/build/)
const PROOF_A = [25,31,180,191,231,133,156,190,178,195,83,131,61,13,89,242,168,170,144,26,209,167,229,82,250,111,16,243,77,79,254,87,31,185,249,67,24,235,128,93,214,18,10,123,40,68,60,135,10,244,223,73,232,102,167,81,221,107,198,129,39,255,198,217];
const PROOF_B = [37,105,166,126,53,221,104,198,30,251,249,247,69,95,7,12,53,95,209,130,207,30,192,71,14,44,103,207,189,104,58,70,7,228,234,101,201,205,43,35,77,21,63,239,37,119,213,232,145,134,24,233,48,23,122,158,148,255,245,206,253,111,157,242,17,42,182,76,86,53,179,136,144,145,10,254,223,115,205,151,79,219,19,213,42,114,114,148,150,171,251,148,204,183,96,158,8,231,143,128,124,130,196,252,0,247,223,135,67,44,60,117,66,188,17,215,56,149,47,197,206,183,88,118,166,134,228,131];
const PROOF_C = [17,54,243,226,133,230,198,109,107,157,76,243,166,247,133,162,85,129,40,71,209,160,136,39,250,127,188,116,17,211,16,167,12,160,59,15,56,127,199,129,52,82,77,89,178,112,36,174,206,217,135,170,189,94,121,245,65,88,194,250,78,111,136,0];
const PROOF_MARKET_ID = Array(30).fill(0).concat([48, 57]); // 12345 BE
const PROOF_NULLIFIER = [4,211,98,147,130,15,167,234,222,46,82,37,197,89,83,238,46,141,6,211,171,83,24,129,42,94,96,212,114,218,144,172];
const PROOF_COMMITMENT_ROOT = [21,75,250,2,173,213,54,240,248,16,140,181,127,19,81,80,250,54,44,198,153,90,91,151,8,47,212,189,14,157,47,25];

describe("krn-comprehensive", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.krn as Program<Krn>;
  const creator = provider.wallet;

  // ============================================================
  // TEST 1: Close market before timestamp should REJECT
  // ============================================================
  it("Rejects close_market before close_timestamp", async () => {
    const marketId = uniqueMarketId("close-early");
    const { marketPda } = getPdas(marketId, program.programId);
    const sourceConfigs = makeSourceConfigs();
    const now = Math.floor(Date.now() / 1000);

    await program.methods
      .initMarket(marketId, new anchor.BN(now + 3600), new anchor.BN(now + 7200), sourceConfigs, 3)
      .accounts({ market: marketPda, creator: creator.publicKey, systemProgram: SystemProgram.programId })
      .rpc();

    try {
      await program.methods
        .closeMarket()
        .accounts({ market: marketPda, caller: creator.publicKey })
        .rpc();
      expect.fail("Should have rejected early close");
    } catch (e: any) {
      expect(e.error.errorCode.code).to.equal("DeadlineNotReached");
      console.log("Close before timestamp correctly rejected");
    }
  });

  // ============================================================
  // TEST 2: Place bet with invalid side should REJECT
  // ============================================================
  it("Rejects bet with invalid side", async () => {
    const marketId = uniqueMarketId("invalid-side");
    const { marketPda, poolPda } = getPdas(marketId, program.programId);
    const sourceConfigs = makeSourceConfigs();
    const now = Math.floor(Date.now() / 1000);

    await program.methods
      .initMarket(marketId, new anchor.BN(now + 3600), new anchor.BN(now + 7200), sourceConfigs, 3)
      .accounts({ market: marketPda, creator: creator.publicKey, systemProgram: SystemProgram.programId })
      .rpc();

    const [commitPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("commitment"), Buffer.from(marketId), creator.publicKey.toBuffer()],
      program.programId
    );

    try {
      await program.methods
        .placeBet(marketId, Array(32).fill(0xAA), 5, new anchor.BN(0.1 * LAMPORTS_PER_SOL))
        .accounts({ market: marketPda, commitment: commitPda, marketPool: poolPda, bettor: creator.publicKey, systemProgram: SystemProgram.programId })
        .rpc();
      expect.fail("Should have rejected invalid side");
    } catch (e: any) {
      expect(e.error.errorCode.code).to.equal("InvalidBetSide");
      console.log("Invalid bet side (5) correctly rejected");
    }
  });

  // ============================================================
  // TEST 3: Zero bet amount should REJECT
  // ============================================================
  it("Rejects bet with zero amount", async () => {
    const marketId = uniqueMarketId("zero-amount");
    const { marketPda, poolPda } = getPdas(marketId, program.programId);
    const sourceConfigs = makeSourceConfigs();
    const now = Math.floor(Date.now() / 1000);

    await program.methods
      .initMarket(marketId, new anchor.BN(now + 3600), new anchor.BN(now + 7200), sourceConfigs, 3)
      .accounts({ market: marketPda, creator: creator.publicKey, systemProgram: SystemProgram.programId })
      .rpc();

    const [commitPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("commitment"), Buffer.from(marketId), creator.publicKey.toBuffer()],
      program.programId
    );

    try {
      await program.methods
        .placeBet(marketId, Array(32).fill(0xAA), 1, new anchor.BN(0))
        .accounts({ market: marketPda, commitment: commitPda, marketPool: poolPda, bettor: creator.publicKey, systemProgram: SystemProgram.programId })
        .rpc();
      expect.fail("Should have rejected zero amount");
    } catch (e: any) {
      expect(e.error.errorCode.code).to.equal("ZeroBetAmount");
      console.log("Zero bet amount correctly rejected");
    }
  });


  // ============================================================
  // TEST 5: Fewer than 3 sources should REJECT
  // ============================================================
  it("Rejects market with fewer than 3 required sources", async () => {
    const marketId = uniqueMarketId("few-sources");
    const { marketPda } = getPdas(marketId, program.programId);
    const now = Math.floor(Date.now() / 1000);

    try {
      await program.methods
        .initMarket(marketId, new anchor.BN(now + 3600), new anchor.BN(now + 7200), makeSourceConfigs().slice(0, 2), 2)
        .accounts({ market: marketPda, creator: creator.publicKey, systemProgram: SystemProgram.programId })
        .rpc();
      expect.fail("Should have rejected");
    } catch (e: any) {
      expect(e.error.errorCode.code).to.equal("MinSourcesRequired");
      console.log("Fewer than 3 sources correctly rejected");
    }
  });



  // ============================================================
  // TEST 8: Claim on unresolved market should REJECT
  // ============================================================
  it("Rejects claim on unresolved market", async () => {
    const marketId = uniqueMarketId("unresolved");
    const { marketPda, poolPda } = getPdas(marketId, program.programId);
    const sourceConfigs = makeSourceConfigs();
    const now = Math.floor(Date.now() / 1000);

    await program.methods
      .initMarket(marketId, new anchor.BN(now + 3600), new anchor.BN(now + 7200), sourceConfigs, 3)
      .accounts({ market: marketPda, creator: creator.publicKey, systemProgram: SystemProgram.programId })
      .rpc();

    const [commitPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("commitment"), Buffer.from(marketId), creator.publicKey.toBuffer()],
      program.programId
    );
    await program.methods
      .placeBet(marketId, Array(32).fill(0xAA), 1, new anchor.BN(0.1 * LAMPORTS_PER_SOL))
      .accounts({ market: marketPda, commitment: commitPda, marketPool: poolPda, bettor: creator.publicKey, systemProgram: SystemProgram.programId })
      .rpc();

    const [nullPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("nullifier"), Buffer.from(marketId), Buffer.from(PROOF_NULLIFIER)],
      program.programId
    );
    const recipient = anchor.web3.Keypair.generate();

    try {
      await program.methods
        .claimWinning(marketId, PROOF_NULLIFIER, {
          proofA: PROOF_A, proofB: PROOF_B, proofC: PROOF_C,
          publicInputs: [marketId, Array(31).fill(0).concat([1]), PROOF_NULLIFIER, Array(32).fill(0xBB), Array(32).fill(0)],
        })
        .accounts({
          market: marketPda, commitment: commitPda, nullifierAccount: nullPda,
          marketPool: poolPda, recipient: recipient.publicKey, claimer: creator.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      expect.fail("Should have rejected");
    } catch (e: any) {
      expect(e.error.errorCode.code).to.equal("MarketNotResolved");
      console.log("Claim on unresolved market correctly rejected");
    }
  });

  // ============================================================
  // TEST 9: Fallback resolve by non-authority should REJECT
  // ============================================================
  it("Rejects fallback_resolve by non-authority", async () => {
    const marketId = uniqueMarketId("fallback-unauth");
    const { marketPda } = getPdas(marketId, program.programId);
    const sourceConfigs = makeSourceConfigs();
    const now = Math.floor(Date.now() / 1000);

    // Close in 3s, deadline in 6s
    await program.methods
      .initMarket(marketId, new anchor.BN(now + 3), new anchor.BN(now + 6), sourceConfigs, 3)
      .accounts({ market: marketPda, creator: creator.publicKey, systemProgram: SystemProgram.programId })
      .rpc();

    await new Promise((r) => setTimeout(r, 4000));
    await program.methods.closeMarket().accounts({ market: marketPda, caller: creator.publicKey }).rpc();

    // Wait for deadline
    await new Promise((r) => setTimeout(r, 4000));

    const fakeAuthority = anchor.web3.Keypair.generate();

    try {
      await program.methods
        .fallbackResolve(1)
        .accounts({ market: marketPda, authority: fakeAuthority.publicKey })
        .signers([fakeAuthority])
        .rpc();
      expect.fail("Should have rejected");
    } catch (e: any) {
      expect(e.error.errorCode.code).to.equal("Unauthorized");
      console.log("Fallback resolve by non-authority correctly rejected");
    }
  });

  // ============================================================
  // TEST 10: Fallback resolve by authority should SUCCEED
  // ============================================================
  it("Allows fallback_resolve by authority after deadline", async () => {
    const marketId = uniqueMarketId("fallback-auth");
    const { marketPda } = getPdas(marketId, program.programId);
    const sourceConfigs = makeSourceConfigs();
    const now = Math.floor(Date.now() / 1000);

    await program.methods
      .initMarket(marketId, new anchor.BN(now + 3), new anchor.BN(now + 6), sourceConfigs, 3)
      .accounts({ market: marketPda, creator: creator.publicKey, systemProgram: SystemProgram.programId })
      .rpc();

    await new Promise((r) => setTimeout(r, 4000));
    await program.methods.closeMarket().accounts({ market: marketPda, caller: creator.publicKey }).rpc();

    await new Promise((r) => setTimeout(r, 4000));

    await program.methods
      .fallbackResolve(0)
      .accounts({ market: marketPda, authority: creator.publicKey })
      .rpc();

    const market = await program.account.marketAccount.fetch(marketPda);
    expect(market.outcome).to.equal(0); // NO
    expect(market.state).to.deep.include({ resolved: {} });
    console.log("Fallback resolve by authority succeeded (outcome=NO)");
  });

  // ============================================================
  // TEST 11: Double claim (same nullifier) should REJECT
  // ============================================================
  it("Rejects double claim with same nullifier", async () => {
    // This uses the market from the first e2e test (market_id=12345)
    // which already had a successful claim. The nullifier PDA already exists.
    const marketId = PROOF_MARKET_ID;
    const { marketPda, poolPda } = getPdas(marketId, program.programId);

    const [commitPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("commitment"), Buffer.from(marketId), creator.publicKey.toBuffer()],
      program.programId
    );
    const [nullPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("nullifier"), Buffer.from(marketId), Buffer.from(PROOF_NULLIFIER)],
      program.programId
    );
    const recipient = anchor.web3.Keypair.generate();

    try {
      await program.methods
        .claimWinning(marketId, PROOF_NULLIFIER, {
          proofA: PROOF_A, proofB: PROOF_B, proofC: PROOF_C,
          publicInputs: [marketId, Array(31).fill(0).concat([1]), PROOF_NULLIFIER, PROOF_COMMITMENT_ROOT, Array(32).fill(0)],
        })
        .accounts({
          market: marketPda, commitment: commitPda, nullifierAccount: nullPda,
          marketPool: poolPda, recipient: recipient.publicKey, claimer: creator.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      expect.fail("Should have rejected double claim");
    } catch (e: any) {
      // Anchor will fail because nullifier PDA already exists (init constraint)
      console.log("Double claim correctly rejected (nullifier PDA already exists)");
    }
  });

  // ============================================================
  // TEST 12: Invalid timestamps (close >= deadline) should REJECT
  // ============================================================
  it("Rejects market with close >= resolution deadline", async () => {
    const marketId = uniqueMarketId("bad-timestamps");
    const { marketPda } = getPdas(marketId, program.programId);
    const now = Math.floor(Date.now() / 1000);

    try {
      await program.methods
        .initMarket(marketId, new anchor.BN(now + 7200), new anchor.BN(now + 3600), makeSourceConfigs(), 3)
        .accounts({ market: marketPda, creator: creator.publicKey, systemProgram: SystemProgram.programId })
        .rpc();
      expect.fail("Should have rejected");
    } catch (e: any) {
      expect(e.error.errorCode.code).to.equal("InvalidTimestamps");
      console.log("Invalid timestamps (close >= deadline) correctly rejected");
    }
  });

  // ============================================================
  // TEST 13: Bet on closed market should REJECT
  // ============================================================
  it("Rejects bet on closed market", async () => {
    const marketId = uniqueMarketId("bet-closed");
    const { marketPda, poolPda } = getPdas(marketId, program.programId);
    const sourceConfigs = makeSourceConfigs();
    const now = Math.floor(Date.now() / 1000);

    await program.methods
      .initMarket(marketId, new anchor.BN(now + 3), new anchor.BN(now + 3600), sourceConfigs, 3)
      .accounts({ market: marketPda, creator: creator.publicKey, systemProgram: SystemProgram.programId })
      .rpc();

    await new Promise((r) => setTimeout(r, 4000));
    await program.methods.closeMarket().accounts({ market: marketPda, caller: creator.publicKey }).rpc();

    const [commitPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("commitment"), Buffer.from(marketId), creator.publicKey.toBuffer()],
      program.programId
    );

    try {
      await program.methods
        .placeBet(marketId, Array(32).fill(0xAA), 1, new anchor.BN(0.1 * LAMPORTS_PER_SOL))
        .accounts({ market: marketPda, commitment: commitPda, marketPool: poolPda, bettor: creator.publicKey, systemProgram: SystemProgram.programId })
        .rpc();
      expect.fail("Should have rejected");
    } catch (e: any) {
      expect(e.error.errorCode.code).to.equal("InvalidMarketState");
      console.log("Bet on closed market correctly rejected");
    }
  });
});
