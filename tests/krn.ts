import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Krn } from "../target/types/krn";
import { expect } from "chai";
import { PublicKey, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { createHash } from "crypto";

function sha256(data: string): number[] {
  const hash = createHash("sha256").update(data).digest();
  return Array.from(hash);
}

describe("krn", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.krn as Program<Krn>;
  const creator = provider.wallet;

  // Generate unique market_id per run to avoid stale PDA conflicts
  const marketId = sha256("test-market-btc-100k-" + Date.now().toString());

  // Source configs (3 sources minimum)
  const sourceConfigs = [
    {
      domainHash: sha256("api.coinbase.com"),
      pathHash: sha256("/v2/prices/BTC-USD/spot"),
      jsonPathHash: sha256("$.data.amount"),
    },
    {
      domainHash: sha256("api.binance.com"),
      pathHash: sha256("/api/v3/ticker/price?symbol=BTCUSDT"),
      jsonPathHash: sha256("$.price"),
    },
    {
      domainHash: sha256("api.kraken.com"),
      pathHash: sha256("/0/public/Ticker?pair=XBTUSD"),
      jsonPathHash: sha256("$.result.XXBTZUSD.c[0]"),
    },
  ];

  // Timestamps: close in 1 hour, deadline in 2 hours
  const now = Math.floor(Date.now() / 1000);
  const closeTimestamp = new anchor.BN(now + 3600);
  const resolutionDeadline = new anchor.BN(now + 7200);

  // Derive PDAs
  const [marketPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("market"), Buffer.from(marketId)],
    program.programId
  );

  const [poolPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("pool"), Buffer.from(marketId)],
    program.programId
  );

  it("Initializes a market", async () => {
    const tx = await program.methods
      .initMarket(
        marketId,
        closeTimestamp,
        resolutionDeadline,
        sourceConfigs,
        3
      )
      .accounts({
        market: marketPda,
        creator: creator.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("init_market tx:", tx);

    // Fetch and verify market account
    const market = await program.account.marketAccount.fetch(marketPda);
    expect(market.state).to.deep.equal({ open: {} });
    expect(market.outcome).to.equal(255);
    expect(market.requiredSources).to.equal(3);
    expect(market.numSources).to.equal(3);
    expect(market.totalPool.toNumber()).to.equal(0);
    expect(market.yesPool.toNumber()).to.equal(0);
    expect(market.noPool.toNumber()).to.equal(0);
    expect(market.sourceCount).to.equal(0);
    expect(market.creator.toBase58()).to.equal(creator.publicKey.toBase58());

    console.log("Market initialized successfully");
  });

  it("Places a YES bet", async () => {
    const betAmount = new anchor.BN(0.5 * LAMPORTS_PER_SOL);
    const commitmentHash = sha256("secret-nonce-bettor1-yes");
    const side = 1; // YES

    const [commitmentPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("commitment"),
        Buffer.from(marketId),
        creator.publicKey.toBuffer(),
      ],
      program.programId
    );

    const tx = await program.methods
      .placeBet(marketId, commitmentHash, side, betAmount)
      .accounts({
        market: marketPda,
        commitment: commitmentPda,
        marketPool: poolPda,
        bettor: creator.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("place_bet tx:", tx);

    // Verify market pool updated
    const market = await program.account.marketAccount.fetch(marketPda);
    expect(market.totalPool.toNumber()).to.equal(betAmount.toNumber());
    expect(market.yesPool.toNumber()).to.equal(betAmount.toNumber());
    expect(market.noPool.toNumber()).to.equal(0);

    // Verify commitment
    const commitment = await program.account.betCommitment.fetch(commitmentPda);
    expect(commitment.side).to.equal(1);
    expect(commitment.amount.toNumber()).to.equal(betAmount.toNumber());
    expect(commitment.claimed).to.equal(false);

    console.log("Bet placed successfully");
  });

  it("Rejects bet with invalid side", async () => {
    const betAmount = new anchor.BN(0.1 * LAMPORTS_PER_SOL);
    const commitmentHash = sha256("secret-nonce-invalid");
    const invalidSide = 5;

    const newBettor = anchor.web3.Keypair.generate();

    // Airdrop to new bettor
    const airdropSig = await provider.connection.requestAirdrop(
      newBettor.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropSig);

    const [commitmentPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("commitment"),
        Buffer.from(marketId),
        newBettor.publicKey.toBuffer(),
      ],
      program.programId
    );

    try {
      await program.methods
        .placeBet(marketId, commitmentHash, invalidSide, betAmount)
        .accounts({
          market: marketPda,
          commitment: commitmentPda,
          marketPool: poolPda,
          bettor: newBettor.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([newBettor])
        .rpc();
      expect.fail("Should have thrown InvalidBetSide error");
    } catch (err) {
      expect(err.toString()).to.include("InvalidBetSide");
      console.log("Invalid side correctly rejected");
    }
  });

  it("Rejects market with fewer than 3 sources", async () => {
    const badMarketId = sha256("bad-market-too-few-sources");
    const twoSources = sourceConfigs.slice(0, 2);

    const [badMarketPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("market"), Buffer.from(badMarketId)],
      program.programId
    );

    try {
      await program.methods
        .initMarket(
          badMarketId,
          closeTimestamp,
          resolutionDeadline,
          twoSources,
          3
        )
        .accounts({
          market: badMarketPda,
          creator: creator.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      expect.fail("Should have thrown MinSourcesRequired error");
    } catch (err) {
      expect(err.toString()).to.include("MinSourcesRequired");
      console.log("Too few sources correctly rejected");
    }
  });
});
