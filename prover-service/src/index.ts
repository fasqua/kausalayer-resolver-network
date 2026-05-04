import { Connection, Keypair } from "@solana/web3.js";
import { Program, AnchorProvider, Wallet } from "@coral-xyz/anchor";
import fs from "fs";
import { RPC_URL, WALLET_PATH, PROGRAM_ID, POLL_INTERVAL } from "./config";
import { findMarketsNeedingProofs } from "./monitor";
import { resolveSource, fetchSourceData } from "./fetcher";
import { generateProof } from "./prover";
import { submitProof, aggregateResolution } from "./submitter";

// Load IDL from anchor build output
const IDL = JSON.parse(
  fs.readFileSync(
    require("path").resolve(__dirname, "../../target/idl/krn.json"),
    "utf8"
  )
);

/**
 * Initialize Anchor program with wallet and RPC connection.
 */
function setupProgram(): { program: Program; payer: Keypair } {
  const walletKey = JSON.parse(fs.readFileSync(WALLET_PATH, "utf8"));
  const payer = Keypair.fromSecretKey(Uint8Array.from(walletKey));

  const connection = new Connection(RPC_URL, "confirmed");
  const wallet = new Wallet(payer);
  const provider = new AnchorProvider(connection, wallet, {
    commitment: "confirmed",
  });

  const program = new Program(IDL, provider);

  console.log("[init] RPC:", RPC_URL);
  console.log("[init] Wallet:", payer.publicKey.toBase58());
  console.log("[init] Program:", PROGRAM_ID.toBase58());

  return { program, payer };
}

/**
 * Process a single market: fetch data, generate proof, submit, aggregate.
 */
async function processMarket(
  program: Program,
  market: any,
  payer: Keypair,
): Promise<void> {
  const marketIdHex = Buffer.from(market.marketId).toString("hex").slice(0, 16);
  console.log("\n[process] Processing market " + marketIdHex + "...");
  console.log("[process] Required sources:", market.requiredSources, "Current:", market.sourceCount);

  for (let i = market.sourceCount; i < market.requiredSources; i++) {
    const sourceConfig = market.sourceConfigs[i];
    if (!sourceConfig) {
      console.log("[process] No source config at index " + i + ", skipping");
      continue;
    }

    // Resolve domain from hash
    const source = resolveSource(sourceConfig.domainHash);
    if (!source) {
      console.log("[process] Unknown source at index " + i + ", skipping");
      continue;
    }

    try {
      // Generate zkTLS proof via Succinct Network
      const proofResult = await generateProof(
        source.domain,
        source.path,
        market.threshold,
        market.comparison,
        source.jsonPath,
      );

      // Submit proof on-chain
      await submitProof(program, market, i, proofResult, payer);
      console.log("[process] Source " + i + " proof submitted");
    } catch (err: any) {
      console.error("[process] Failed for source " + i + ":", err.message);
      // Continue with next source, don't abort entire market
    }
  }

  // Re-fetch market to check if we have enough proofs now
  try {
    const [marketPda] = require("@solana/web3.js").PublicKey.findProgramAddressSync(
      [Buffer.from("market"), Buffer.from(market.marketId)],
      program.programId
    );
    const updatedMarket = await (program.account as any).marketAccount.fetch(marketPda);
    const updated = updatedMarket as any;

    if (updated.sourceCount >= updated.requiredSources) {
      console.log("[process] Enough proofs, aggregating resolution...");
      await aggregateResolution(program, market, payer);
      console.log("[process] Market " + marketIdHex + " resolved");
    }
  } catch (err: any) {
    console.error("[process] Aggregation failed:", err.message);
  }
}

/**
 * Main polling loop.
 */
async function main(): Promise<void> {
  console.log("=== KRN Prover Service ===");
  console.log("Automated market resolution via zkTLS proofs\n");

  const { program, payer } = setupProgram();

  console.log("[main] Polling every " + (POLL_INTERVAL / 1000) + " seconds\n");

  while (true) {
    try {
      const markets = await findMarketsNeedingProofs(program);

      for (const market of markets) {
        await processMarket(program, market, payer);
      }
    } catch (err: any) {
      console.error("[main] Poll error:", err.message);
    }

    // Wait before next poll
    await new Promise((resolve) => setTimeout(resolve, POLL_INTERVAL));
  }
}

main().catch((err) => {
  console.error("[fatal]", err);
  process.exit(1);
});
