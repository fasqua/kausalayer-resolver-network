import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Krn } from "../target/types/krn";
import { assert } from "chai";
import { PublicKey, SystemProgram, ComputeBudgetProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { createHash } from "crypto";
import { buildPoseidon } from "circomlibjs";
import * as path from "path";

const snarkjs = require("snarkjs");

function sha256(data: string): number[] {
  return Array.from(createHash("sha256").update(data).digest());
}

const BN254_PRIME = BigInt("21888242871839275222246405745257275088548364400416034343698204186575808495617");

let poseidonHasher: any;

function poseidonHash(inputs: bigint[]): Uint8Array {
  const hash = poseidonHasher(inputs);
  const hashBigInt = poseidonHasher.F.toObject(hash);
  const hex = hashBigInt.toString(16).padStart(64, "0");
  const bytes = new Uint8Array(32);
  for (let i = 0; i < 64; i += 2) {
    bytes[i / 2] = parseInt(hex.substring(i, i + 2), 16);
  }
  return bytes;
}

function poseidonHashBigInt(inputs: bigint[]): bigint {
  const hash = poseidonHasher(inputs);
  return poseidonHasher.F.toObject(hash);
}

function bigintToBytes32(val: bigint): number[] {
  const hex = val.toString(16).padStart(64, "0");
  const bytes: number[] = [];
  for (let i = 0; i < 64; i += 2) {
    bytes.push(parseInt(hex.substring(i, i + 2), 16));
  }
  return bytes;
}

function bytes32ToBigInt(bytes: number[] | Uint8Array): bigint {
  let hex = "0x";
  for (const b of bytes) {
    hex += b.toString(16).padStart(2, "0");
  }
  return BigInt(hex);
}

function computeZeroHashes(depth: number): bigint[] {
  const zeros: bigint[] = new Array(depth);
  zeros[0] = BigInt(0);
  for (let i = 1; i < depth; i++) {
    zeros[i] = poseidonHashBigInt([zeros[i - 1], zeros[i - 1]]);
  }
  return zeros;
}

function computeMerkleProof(
  leaf: bigint,
  index: number,
  depth: number
): { root: bigint; siblings: bigint[]; directions: number[] } {
  const zeroHashes = computeZeroHashes(depth);
  const siblings: bigint[] = [];
  const directions: number[] = [];
  let currentHash = leaf;
  let currentIndex = index;
  for (let level = 0; level < depth; level++) {
    if (currentIndex % 2 === 0) {
      siblings.push(zeroHashes[level]);
      directions.push(0);
      currentHash = poseidonHashBigInt([currentHash, zeroHashes[level]]);
    } else {
      throw new Error("Multi-bet Merkle proof not implemented for this test");
    }
    currentIndex = Math.floor(currentIndex / 2);
  }
  return { root: currentHash, siblings, directions };
}

describe("KRN E2E Sports: PHI vs BOS (Head-to-Head)", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.krn as Program<Krn>;
  const creator = provider.wallet;

  const secretNonce = BigInt(987654321);
  const originalPubkey = BigInt("8888888888888888888");
  const betAmount = 0.5 * LAMPORTS_PER_SOL;
  const betSide = 1; // YES -- PHI wins
  const resolvedOutcome = 1; // YES

  const marketId = sha256("sports-phi-bos-" + Date.now().toString());
  const marketIdBigInt = bytes32ToBigInt(marketId);
  const marketIdField = marketIdBigInt % BN254_PRIME;

  const jsonPath = "events.0.competitions.0.competitors.1.score|events.0.competitions.0.competitors.0.score";

  const sourceConfigs = [
    {
      domainHash: sha256("site.api.espn.com"),
      pathHash: sha256("/apis/site/v2/sports/basketball/nba/scoreboard"),
      jsonPathHash: sha256(jsonPath),
    },
  ];

  const proofHex = "4388a21c0000000000000000000000000000000000000000000000000000000000000000002f850ee998974d6cc00e50cd0814b098c05bfade466d28573240d057f2535200000000000000000000000000000000000000000000000000000000000000000cb6398270f6edc8bf0ec76edb6f82bf0d17d0fd9a6222ddae444df92163757c220b6a9f3e43def88d4f55a2ab1202cd25e7b04a61a05813a089ce0351345cbe23fc8984689154fe7d539bad6276ea9fb5dcc5a8de2701ecb27360c3e564b7291a12dcb1d19e34319522efbdeed251e4dd9f8fcfb208e668fe9efaf1a1089d2c2b03f30b84ddf151a4a1353e459b9907dbae67a74b181a0f1d0bf828f991883f2b3311a43721ba76591f7b5c94c9bd8808eecf3d30cbdf950a076b02baaf70fe2b3a750f6676893c4f45c7f55d3a498eea3318b3ef7c0363e329b57bca7154870ad6fb8ad8143e571d6266500507be3ae9c3cfe2add5af68ba5b4f792f9ab33f";
  const pvHex = "013ce161791900b9fc5c0900f0a29415af478e294a2b4a447640cef2dfb03c4489bde461a2ce18e7299bb35a9e844729bbd237bd3395222290dd7278aa3ddd389cc1e1d165cc4bafe5e19cb336d24b30c013e7bdb2e93659d6086672be7191a02262a7e032ceb43fc9000000000000000100000000724b524e0201030000000000000000000000000000006d11736974652e6170692e6573706e2e636f6d576576656e74732e302e636f6d7065746974696f6e732e302e636f6d70657469746f72732e312e73636f72657c6576656e74732e302e636f6d7065746974696f6e732e302e636f6d70657469746f72732e302e73636f7265";
  const proof = Buffer.from(proofHex, "hex");
  const publicValues = Buffer.from(pvHex, "hex");

  const [marketPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("market"), Buffer.from(marketId)],
    program.programId
  );
  const [poolPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("pool"), Buffer.from(marketId)],
    program.programId
  );

  before(async () => {
    poseidonHasher = await buildPoseidon();
  });

  it("Step 1: Init sports market (PHI vs BOS, 1 ESPN source, comparison=first_greater)", async () => {
    const now = Math.floor(Date.now() / 1000);
    const threshold = new anchor.BN(0);
    const comparison = 3; // first_greater

    await program.methods
      .initMarket(marketId, new anchor.BN(now + 3), new anchor.BN(now + 3600), sourceConfigs, 1, threshold, comparison)
      .accounts({ market: marketPda, creator: creator.publicKey, systemProgram: SystemProgram.programId })
      .rpc();

    console.log("   Sports market initialized: PHI vs BOS (head-to-head)");
  });

  it("Step 2: Place bet (YES = PHI wins, 0.5 SOL)", async () => {
    const commitmentHash = poseidonHash([
      marketIdField,
      BigInt(resolvedOutcome),
      BigInt(betAmount),
      secretNonce,
      originalPubkey,
    ]);

    const [commitmentPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("commitment"), Buffer.from(marketId), creator.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .placeBet(marketId, Array.from(commitmentHash), betSide, new anchor.BN(betAmount))
      .accounts({
        market: marketPda,
        commitment: commitmentPda,
        marketPool: poolPda,
        bettor: creator.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("   Bet placed: YES (PHI wins), 0.5 SOL");
  });

  it("Step 3: Close market", async () => {
    await new Promise((r) => setTimeout(r, 4000));
    await program.methods
      .closeMarket()
      .accounts({ market: marketPda, caller: creator.publicKey })
      .rpc();
    console.log("   Market closed");
  });

  it("Step 4: Submit ESPN zkTLS proof (comparison=3, head-to-head)", async () => {
    const [proofPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("proof"), Buffer.from(marketId), Buffer.from([0])],
      program.programId
    );

    await program.methods
      .submitProof(marketId, 0, { proof, publicValues })
      .accounts({
        market: marketPda,
        proofSubmission: proofPda,
        submitter: creator.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 })])
      .rpc();

    const market = await program.account.marketAccount.fetch(marketPda);
    console.log("   ESPN proof submitted. sourceCount:", market.sourceCount);
  });

  it("Step 5: Aggregate resolution (PHI wins)", async () => {
    const [proofPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("proof"), Buffer.from(marketId), Buffer.from([0])],
      program.programId
    );

    await program.methods
      .aggregateResolution()
      .accounts({
        market: marketPda,
        caller: creator.publicKey,
      })
      .remainingAccounts([{
        pubkey: proofPda,
        isSigner: false,
        isWritable: false,
      }])
      .rpc();

    const market = await program.account.marketAccount.fetch(marketPda);
    assert.deepInclude(market.state, { resolved: {} });
    assert.equal(market.outcome, 1); // YES
    console.log("   Market resolved: PHI wins (outcome=YES)");
  });

  it("Step 6: Claim winning with ownership proof", async () => {
    const TREE_DEPTH = 10;

    const commitmentHashBigInt = poseidonHashBigInt([
      marketIdField,
      BigInt(resolvedOutcome),
      BigInt(betAmount),
      secretNonce,
      originalPubkey,
    ]);

    const merkleProof = computeMerkleProof(commitmentHashBigInt, 0, TREE_DEPTH);

    const market = await program.account.marketAccount.fetch(marketPda);
    const onChainRoot = bytes32ToBigInt(market.commitmentRoot);
    assert.equal(merkleProof.root.toString(16), onChainRoot.toString(16), "Merkle root mismatch");

    const nullifierBigInt = poseidonHashBigInt([marketIdField, secretNonce, originalPubkey]);
    const nullifierBytes = bigintToBytes32(nullifierBigInt);

    const circuitInput = {
      market_id: marketIdField.toString(),
      resolved_outcome: resolvedOutcome.toString(),
      nullifier: nullifierBigInt.toString(),
      commitment_root: merkleProof.root.toString(),
      amount: betAmount.toString(),
      secret_nonce: secretNonce.toString(),
      original_pubkey: originalPubkey.toString(),
      sibling: merkleProof.siblings.map((s) => s.toString()),
      direction: merkleProof.directions.map((d) => d.toString()),
    };

    const wasmPath = path.resolve(__dirname, "../circuits/ownership/build/ownership_js/ownership.wasm");
    const zkeyPath = path.resolve(__dirname, "../circuits/ownership/build/ownership_final.zkey");

    const { proof: snarkProof, publicSignals } = await snarkjs.groth16.fullProve(
      circuitInput,
      wasmPath,
      zkeyPath
    );

    const vkPath = path.resolve(__dirname, "../circuits/ownership/build/verification_key.json");
    const vk = JSON.parse(require("fs").readFileSync(vkPath, "utf8"));
    const isValid = await snarkjs.groth16.verify(vk, publicSignals, snarkProof);
    assert.isTrue(isValid, "Local proof verification failed");

    const FIELD_PRIME = BigInt("21888242871839275222246405745257275088696311157297823662689037894645226208583");
    const ax = BigInt(snarkProof.pi_a[0]);
    const ay = BigInt(snarkProof.pi_a[1]);
    const negAy = FIELD_PRIME - ay;
    const proofA = [...bigintToBytes32(ax), ...bigintToBytes32(negAy)];

    const bx0 = BigInt(snarkProof.pi_b[0][0]);
    const bx1 = BigInt(snarkProof.pi_b[0][1]);
    const by0 = BigInt(snarkProof.pi_b[1][0]);
    const by1 = BigInt(snarkProof.pi_b[1][1]);
    const proofB = [
      ...bigintToBytes32(bx1), ...bigintToBytes32(bx0),
      ...bigintToBytes32(by1), ...bigintToBytes32(by0),
    ];

    const cx = BigInt(snarkProof.pi_c[0]);
    const cy = BigInt(snarkProof.pi_c[1]);
    const proofC = [...bigintToBytes32(cx), ...bigintToBytes32(cy)];

    const publicInputs = publicSignals.map((s: string) => bigintToBytes32(BigInt(s)));

    const ownershipProof = { proofA, proofB, proofC, publicInputs };

    const [commitmentPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("commitment"), Buffer.from(marketId), creator.publicKey.toBuffer()],
      program.programId
    );

    const [nullifierPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("nullifier"), Buffer.from(marketId), Buffer.from(nullifierBytes)],
      program.programId
    );

    const recipient = anchor.web3.Keypair.generate();

    const claimTx = await program.methods
      .claimWinning(marketId, nullifierBytes, ownershipProof)
      .accounts({
        market: marketPda,
        commitment: commitmentPda,
        nullifierAccount: nullifierPda,
        marketPool: poolPda,
        recipient: recipient.publicKey,
        claimer: creator.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 })])
      .rpc();

    const recipientBalance = await provider.connection.getBalance(recipient.publicKey);
    assert.isAbove(recipientBalance, 0, "Recipient should have received payout");

    console.log("   Claim TX:", claimTx);
    console.log("   Recipient balance:", recipientBalance / LAMPORTS_PER_SOL, "SOL");
    console.log("\n=== SPORTS E2E COMPLETE ===");
    console.log("PHI 109 vs BOS 100 -> PHI wins -> bettor claimed 0.5 SOL");
    console.log("ESPN API -> SP1 zkTLS -> Groth16 on-chain -> ZK ownership claim");
  });
});
