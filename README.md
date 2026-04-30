# KausaLayer Resolver Network (KRN)

Resolution infrastructure for prediction markets on Solana. Uses zkTLS proof verification (Reclaim Protocol) and ZK ownership claims (Groth16) for trustless market resolution with bettor privacy.

## What KRN Does

Prediction markets need a way to determine outcomes. Current solutions rely on human voting (UMA/optimistic oracles), which is slow and vulnerable to manipulation. KRN replaces human voters with cryptographic verification:

1. **zkTLS Proofs** — Bots fetch outcomes from data sources (Coinbase, ESPN, etc.) and generate zkTLS proofs that the data is authentic. Multiple sources required for consensus.
2. **On-Chain Verification** — Program verifies proof signatures from Reclaim Protocol attestors using secp256k1 recovery + keccak256.
3. **Private Claims** — Winners prove bet ownership with Groth16 ZK proofs (Poseidon commitments + nullifiers) and claim to a fresh address. No link between betting and claiming address.

## Project Structure

```
kausalayer-resolver-network/
|-- programs/krn/              # Anchor program
|   |-- src/
|       |-- state/             # MarketAccount, ProofSubmission, BetCommitment, NullifierAccount
|       |-- instructions/      # 8 instructions + verifier module
|       |-- errors/            # 19 error codes
|-- circuits/ownership/        # Circom Groth16 ownership circuit (PoC)
|-- prover/reclaim/            # Reclaim zkFetch integration
|-- tests/                     # Anchor integration tests
```

## Program

**Program ID (localnet):** `5qkQX3VaiUni5xLA7HQGbGPCPLajbELoj5QAA2PbnFDK`

| Instruction | Description |
|---|---|
| init_market | Create market with source configs (min 3 sources) |
| place_bet | Place bet with Poseidon commitment hash |
| close_market | Close betting after deadline |
| submit_proof | Submit zkTLS proof from Reclaim attestor |
| aggregate_resolution | Aggregate proofs, resolve by majority vote |
| claim_winning | Claim winnings with ZK ownership proof |
| fallback_resolve | Manual resolution by authority (fallback) |
| refund_market | Refund bettors on failed market |

**Accounts:** MarketAccount (1144 bytes), ProofSubmission (116 bytes), BetCommitment (83 bytes), NullifierAccount (81 bytes)

## On-Chain Verification

**zkTLS (Reclaim Protocol):** Signature verification using secp256k1_recover + keccak256. Follows the exact verification logic of Reclaim JS SDK — reconstruct signed message from claim data, apply Ethereum personal_sign prefix, recover signer address, check against known attestor.

**Ownership (Groth16 BN254):** Circuit with 1076 constraints. Proves: Poseidon commitment reconstruction, Merkle inclusion, nullifier correctness. Bettor claims to fresh address without revealing betting identity.

## Development

### Prerequisites

- Rust 1.91+
- Solana CLI 3.1+
- Anchor 0.32.1
- Node.js 24+
- Circom 2.2+ (for circuit development)
- snarkjs 0.7+ (for proof generation)

### Build and Test

```bash
anchor build
anchor test
```

### Ownership Circuit

```bash
cd circuits/ownership
npm install
mkdir -p build
circom ownership.circom --r1cs --wasm --sym -o build/
node compute-input.js
node build/ownership_js/generate_witness.js build/ownership_js/ownership.wasm build/input.json build/witness.wtns
snarkjs groth16 prove build/ownership_final.zkey build/witness.wtns build/proof.json build/public.json
snarkjs groth16 verify build/verification_key.json build/public.json build/proof.json
```

### zkFetch Integration

```bash
cd prover/reclaim
npm install
echo "RECLAIM_APP_ID=your_app_id" > .env
echo "RECLAIM_APP_SECRET=your_app_secret" >> .env
node test-zkfetch.js
```

## Current Status

- [x] Anchor program with 8 instructions
- [x] 4 account structures with proper sizing
- [x] 19 custom error codes
- [x] 4 integration tests passing
- [x] Reclaim zkFetch proof generation confirmed
- [x] On-chain verifier (secp256k1 + keccak256)
- [x] Ownership circuit PoC (Groth16, 1076 constraints)
- [ ] End-to-end devnet testing with sample markets
- [ ] Benchmark dashboard
- [ ] Devnet public launch

## Tech Stack

- **Program:** Rust, Anchor 0.32.1, Solana
- **zkTLS:** Reclaim Protocol zkFetch SDK
- **Ownership Circuit:** Circom 2.2, snarkjs, Groth16 BN254
- **Cryptography:** Poseidon hash, secp256k1, keccak256

## Links

- Website: [kausalayer.com](https://kausalayer.com)
- X: [@kausalayer](https://x.com/kausalayer)

## License

Apache 2.0
