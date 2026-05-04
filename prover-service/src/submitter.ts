import { PublicKey, SystemProgram, ComputeBudgetProgram } from "@solana/web3.js";
import { Program } from "@coral-xyz/anchor";
import { COMPUTE_UNITS } from "./config";
import { ClosedMarket } from "./monitor";
import { ProofResult } from "./prover";

/**
 * Submit a zkTLS proof on-chain for a specific source index.
 */
export async function submitProof(
  program: Program,
  market: ClosedMarket,
  sourceIndex: number,
  proofResult: ProofResult,
  payer: any,
): Promise<string> {
  const marketId = market.marketId;

  const [marketPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("market"), Buffer.from(marketId)],
    program.programId
  );

  const [proofPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("proof"), Buffer.from(marketId), Buffer.from([sourceIndex])],
    program.programId
  );

  console.log("[submitter] Submitting proof for source_index=" + sourceIndex);

  const tx = await program.methods
    .submitProof(
      marketId,
      sourceIndex,
      {
        proof: proofResult.proof,
        publicValues: proofResult.publicValues,
      }
    )
    .accounts({
      market: marketPda,
      proofSubmission: proofPda,
      submitter: payer.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .preInstructions([
      ComputeBudgetProgram.setComputeUnitLimit({ units: COMPUTE_UNITS }),
    ])
    .rpc();

  console.log("[submitter] Proof submitted: tx=" + tx);
  return tx;
}

/**
 * Call aggregate_resolution after enough proofs have been submitted.
 * Passes all proof PDAs as remaining accounts for vote counting.
 */
export async function aggregateResolution(
  program: Program,
  market: ClosedMarket,
  payer: any,
): Promise<string> {
  const marketId = market.marketId;

  const [marketPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("market"), Buffer.from(marketId)],
    program.programId
  );

  // Build remaining accounts: all proof PDAs for this market
  const proofPdas = [];
  for (let i = 0; i < market.numSources; i++) {
    const [proofPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("proof"), Buffer.from(marketId), Buffer.from([i])],
      program.programId
    );
    proofPdas.push({
      pubkey: proofPda,
      isSigner: false,
      isWritable: false,
    });
  }

  console.log("[submitter] Aggregating resolution with " + proofPdas.length + " proof accounts");

  const tx = await program.methods
    .aggregateResolution()
    .accounts({
      market: marketPda,
      caller: payer.publicKey,
    })
    .remainingAccounts(proofPdas)
    .rpc();

  console.log("[submitter] Market resolved: tx=" + tx);
  return tx;
}
