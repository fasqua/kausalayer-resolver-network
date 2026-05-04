import { Connection, PublicKey } from "@solana/web3.js";
import { Program, AnchorProvider } from "@coral-xyz/anchor";
import { PROGRAM_ID } from "./config";

// MarketState enum values from on-chain
const MARKET_STATE_CLOSED = 1;
const MARKET_STATE_RESOLVING = 2;

export interface ClosedMarket {
  publicKey: PublicKey;
  marketId: number[];
  requiredSources: number;
  sourceCount: number;
  numSources: number;
  sourceConfigs: Array<{
    domainHash: number[];
    pathHash: number[];
    jsonPathHash: number[];
  }>;
  threshold: bigint;
  comparison: number;
  resolutionDeadline: number;
}

/**
 * Scan all MarketAccount PDAs and return markets that need proof submission.
 * Targets markets in Closed state (no proofs yet) or Resolving state
 * (some proofs submitted but not enough).
 */
export async function findMarketsNeedingProofs(
  program: Program
): Promise<ClosedMarket[]> {
  const markets: ClosedMarket[] = [];

  try {
    let allMarkets: any[] = [];
    try {
      allMarkets = await (program.account as any).marketAccount.all();
    } catch (err: any) {
      // Fallback: fetch with getProgramAccounts and deserialize individually
      console.log("[monitor] Bulk fetch failed, trying individual deserialization...");
      const connection = program.provider.connection;
      const accounts = await connection.getProgramAccounts(program.programId);
      for (const acc of accounts) {
        try {
          const decoded = program.coder.accounts.decode("marketAccount", acc.account.data);
          allMarkets.push({ publicKey: acc.pubkey, account: decoded });
        } catch {
          // Skip accounts that can't be decoded (old/resized)
        }
      }
    }

    for (const account of allMarkets) {
      const market = account.account as any;
      const state = market.state;

      // Check if market is Closed or Resolving with insufficient proofs
      const isClosed = state.closed !== undefined;
      const isResolving = state.resolving !== undefined;

      if (!isClosed && !isResolving) continue;

      // Skip if already has enough proofs
      if (market.sourceCount >= market.requiredSources) continue;

      // Skip if past resolution deadline
      const now = Math.floor(Date.now() / 1000);
      if (now >= market.resolutionDeadline.toNumber()) {
        console.log(`[monitor] Skipping expired market: ${account.publicKey.toBase58()}`);
        continue;
      }

      markets.push({
        publicKey: account.publicKey,
        marketId: Array.from(market.marketId),
        requiredSources: market.requiredSources,
        sourceCount: market.sourceCount,
        numSources: market.numSources,
        sourceConfigs: market.sourceConfigs.slice(0, market.numSources).map((sc: any) => ({
          domainHash: Array.from(sc.domainHash),
          pathHash: Array.from(sc.pathHash),
          jsonPathHash: Array.from(sc.jsonPathHash),
        })),
        threshold: BigInt(market.threshold.toString()),
        comparison: market.comparison,
        resolutionDeadline: market.resolutionDeadline.toNumber(),
      });
    }

    if (markets.length > 0) {
      console.log(`[monitor] Found ${markets.length} market(s) needing proofs`);
    }
  } catch (err) {
    console.error("[monitor] Error scanning markets:", err);
  }

  return markets;
}
