import { PublicKey } from "@solana/web3.js";
import dotenv from "dotenv";
import path from "path";

dotenv.config({ path: path.resolve(__dirname, "../.env") });

// Solana RPC
export const RPC_URL = process.env.RPC_URL || "https://api.devnet.solana.com";

// Wallet path for signing transactions
export const WALLET_PATH = process.env.WALLET_PATH || path.resolve(process.env.HOME || "~", "krn-devnet-wallet.json");

// KRN Program ID
export const PROGRAM_ID = new PublicKey("5qkQX3VaiUni5xLA7HQGbGPCPLajbELoj5QAA2PbnFDK");

// Succinct Prover Network
export const SUCCINCT_API_URL = process.env.SUCCINCT_API_URL || "https://prover.succinct.xyz/api";
export const SUCCINCT_API_KEY = process.env.SUCCINCT_API_KEY || "";

// Polling interval in milliseconds (default 30 seconds)
export const POLL_INTERVAL = parseInt(process.env.POLL_INTERVAL || "30000", 10);

// Compute budget for proof submission (1.4M CU)
export const COMPUTE_UNITS = 1_400_000;

// Logging
export const LOG_LEVEL = process.env.LOG_LEVEL || "info";
