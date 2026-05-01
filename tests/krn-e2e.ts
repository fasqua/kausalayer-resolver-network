import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Krn } from "../target/types/krn";
import { expect } from "chai";
import { PublicKey, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { ethers } from "ethers";

// Set TEST_ATTESTOR_KEY env var to run (matches test-mode RECLAIM_ATTESTOR in verifier.rs)
const TEST_ATTESTOR_KEY = process.env.TEST_ATTESTOR_KEY || "";

function toBytes32BE(decStr: string): number[] {
  const bn = BigInt(decStr);
  const hex = bn.toString(16).padStart(64, "0");
  const bytes: number[] = [];
  for (let i = 0; i < 64; i += 2) {
    bytes.push(parseInt(hex.substring(i, i + 2), 16));
  }
  return bytes;
}

function hexToBytes(hex: string): number[] {
  const clean = hex.startsWith("0x") ? hex.slice(2) : hex;
  const bytes: number[] = [];
  for (let i = 0; i < clean.length; i += 2) {
    bytes.push(parseInt(clean.substring(i, i + 2), 16));
  }
  return bytes;
}

async function signReclaimProof(
  identifier: number[],
  owner: number[],
  timestampS: number,
  epoch: number
): Promise<{ signature: number[]; recoveryId: number }> {
  const wallet = new ethers.Wallet(TEST_ATTESTOR_KEY);

  const identifierHex = "0x" + Buffer.from(identifier).toString("hex");
  const ownerHex = "0x" + Buffer.from(owner).toString("hex");

  const message = `${identifierHex}\n${ownerHex}\n${timestampS}\n${epoch}`;

  // ethers v5 signMessage applies EIP-191 prefix automatically
  const sig = await wallet.signMessage(message);
  const sigBytes = ethers.utils.arrayify(sig);

  // Split: r(32) + s(32) + v(1)
  const r = Array.from(sigBytes.slice(0, 32));
  const s = Array.from(sigBytes.slice(32, 64));
  const v = sigBytes[64];
  const recoveryId = v - 27;

  return { signature: [...r, ...s], recoveryId };
}

describe("krn-e2e-groth16", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.krn as Program<Krn>;
  const creator = provider.wallet;

  // market_id = 12345 as [u8; 32] big-endian (matches circuit input)
  const marketId = Array(30).fill(0).concat([48, 57]);

  // Source configs (3 sources minimum)
  const sourceConfigs = [
    { domainHash: Array(32).fill(1), pathHash: Array(32).fill(2), jsonPathHash: Array(32).fill(3) },
    { domainHash: Array(32).fill(4), pathHash: Array(32).fill(5), jsonPathHash: Array(32).fill(6) },
    { domainHash: Array(32).fill(7), pathHash: Array(32).fill(8), jsonPathHash: Array(32).fill(9) },
  ];

  // commitment_root from circuit proof (public.json output)
  const commitmentRoot = [21,75,250,2,173,213,54,240,248,16,140,181,127,19,81,80,250,54,44,198,153,90,91,151,8,47,212,189,14,157,47,25];

  // Groth16 proof bytes (pre-serialized from proof.json, proof_a negated, proof_b swapped)
  const proofA = [25,31,180,191,231,133,156,190,178,195,83,131,61,13,89,242,168,170,144,26,209,167,229,82,250,111,16,243,77,79,254,87,31,185,249,67,24,235,128,93,214,18,10,123,40,68,60,135,10,244,223,73,232,102,167,81,221,107,198,129,39,255,198,217];
  const proofB = [37,105,166,126,53,221,104,198,30,251,249,247,69,95,7,12,53,95,209,130,207,30,192,71,14,44,103,207,189,104,58,70,7,228,234,101,201,205,43,35,77,21,63,239,37,119,213,232,145,134,24,233,48,23,122,158,148,255,245,206,253,111,157,242,17,42,182,76,86,53,179,136,144,145,10,254,223,115,205,151,79,219,19,213,42,114,114,148,150,171,251,148,204,183,96,158,8,231,143,128,124,130,196,252,0,247,223,135,67,44,60,117,66,188,17,215,56,149,47,197,206,183,88,118,166,134,228,131];
  const proofC = [17,54,243,226,133,230,198,109,107,157,76,243,166,247,133,162,85,129,40,71,209,160,136,39,250,127,188,116,17,211,16,167,12,160,59,15,56,127,199,129,52,82,77,89,178,112,36,174,206,217,135,170,189,94,121,245,65,88,194,250,78,111,136,0];

  // Public inputs from proof
  const nullifier = [4,211,98,147,130,15,167,234,222,46,82,37,197,89,83,238,46,141,6,211,171,83,24,129,42,94,96,212,114,218,144,172];

  // PDAs
  const [marketPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("market"), Buffer.from(marketId)],
    program.programId
  );
  const [poolPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("pool"), Buffer.from(marketId)],
    program.programId
  );

  it("E2E: init -> bet -> proofs -> resolve -> claim with Groth16", async () => {
    // === Step 1: Initialize market ===
    const now = Math.floor(Date.now() / 1000);
    const closeTimestamp = new anchor.BN(now + 5);
    const resolutionDeadline = new anchor.BN(now + 3600);

    const initTx = await program.methods
      .initMarket(marketId, closeTimestamp, resolutionDeadline, sourceConfigs, 3)
      .accounts({
        market: marketPda,
        creator: creator.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("1. init_market tx:", initTx);

    const market = await program.account.marketAccount.fetch(marketPda);
    expect(market.state).to.deep.include({ open: {} });
    console.log("   Market initialized (market_id=12345)");

    // === Step 2: Place bet with commitment_root matching proof ===
    const betAmount = new anchor.BN(0.5 * LAMPORTS_PER_SOL);
    const commitmentHash = Array(32).fill(0xAB);
    const side = 1; // YES

    const [commitmentPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("commitment"), Buffer.from(marketId), creator.publicKey.toBuffer()],
      program.programId
    );

    const betTx = await program.methods
      .placeBet(marketId, commitmentHash, commitmentRoot, side, betAmount)
      .accounts({
        market: marketPda,
        commitment: commitmentPda,
        marketPool: poolPda,
        bettor: creator.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("2. place_bet tx:", betTx);

    const marketAfterBet = await program.account.marketAccount.fetch(marketPda);
    expect(Array.from(marketAfterBet.commitmentRoot)).to.deep.equal(commitmentRoot);
    console.log("   commitment_root stored on-chain matches proof");

    // === Step 3: Wait for close timestamp, then close market ===
    console.log("   Waiting for close timestamp...");
    await new Promise((r) => setTimeout(r, 6000));

    const closeTx = await program.methods
      .closeMarket()
      .accounts({
        market: marketPda,
        caller: creator.publicKey,
      })
      .rpc();
    console.log("3. close_market tx:", closeTx);

    // === Step 4: Submit 3 Reclaim proofs with test attestor signature ===
    const proofSubmissions: PublicKey[] = [];
    const ownerBytes = Array(20).fill(0x01); // dummy owner
    const timestampS = Math.floor(Date.now() / 1000);

    for (let i = 0; i < 3; i++) {
      const [proofPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("proof"), Buffer.from(marketId), Buffer.from([i])],
        program.programId
      );
      proofSubmissions.push(proofPda);

      // Create identifier from source config
      const identifier = sourceConfigs[i].domainHash;

      // Sign with test attestor
      const { signature, recoveryId } = await signReclaimProof(
        identifier, ownerBytes, timestampS, i
      );

      const reclaimProof = {
        identifier,
        owner: ownerBytes,
        timestampS: new anchor.BN(timestampS),
        epoch: i,
        signature,
        recoveryId,
      };

      const submitTx = await program.methods
        .submitProof(marketId, i, 1, reclaimProof)
        .accounts({
          market: marketPda,
          proofSubmission: proofPda,
          submitter: creator.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      console.log(`4.${i + 1} submit_proof[${i}] tx:`, submitTx);
    }

    // === Step 5: Aggregate resolution ===
    const aggTx = await program.methods
      .aggregateResolution()
      .accounts({
        market: marketPda,
        caller: creator.publicKey,
      })
      .remainingAccounts(
        proofSubmissions.map((pk) => ({
          pubkey: pk,
          isWritable: false,
          isSigner: false,
        }))
      )
      .rpc();
    console.log("5. aggregate_resolution tx:", aggTx);

    const resolvedMarket = await program.account.marketAccount.fetch(marketPda);
    expect(resolvedMarket.outcome).to.equal(1);
    expect(resolvedMarket.state).to.deep.include({ resolved: {} });
    console.log("   Market resolved: outcome=YES");

    // === Step 6: Claim winning with real Groth16 proof ===
    const [nullifierPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("nullifier"), Buffer.from(marketId), Buffer.from(nullifier)],
      program.programId
    );

    const recipient = anchor.web3.Keypair.generate();

    const ownershipProof = {
      proofA,
      proofB,
      proofC,
      publicInputs: [
        marketId,
        Array(31).fill(0).concat([1]),
        nullifier,
        commitmentRoot,
      ],
    };

    const claimTx = await program.methods
      .claimWinning(marketId, nullifier, ownershipProof)
      .accounts({
        market: marketPda,
        commitment: commitmentPda,
        nullifierAccount: nullifierPda,
        marketPool: poolPda,
        recipient: recipient.publicKey,
        claimer: creator.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("6. claim_winning tx:", claimTx);
    console.log("\n=== E2E GROTH16 + RECLAIM PROOF VERIFICATION SUCCESS ON DEVNET ===");
  });
});
