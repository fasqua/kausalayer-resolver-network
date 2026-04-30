pragma circom 2.1.0;

include "node_modules/circomlib/circuits/poseidon.circom";
include "node_modules/circomlib/circuits/mux1.circom";

/// Bettor Ownership Circuit for KRN
///
/// Proves that the claimer owns a winning bet commitment without
/// revealing the original betting address.
///
/// Public inputs:
///   - market_id: ID of the resolved market
///   - resolved_outcome: the winning outcome (0=NO, 1=YES)
///   - nullifier: unique hash to prevent double-claiming
///   - commitment_root: Merkle root of all commitments (simplified for PoC)
///
/// Private inputs (witnesses):
///   - secret_nonce: random nonce used when placing bet
///   - amount: bet amount
///   - original_pubkey: pubkey used when placing bet (field element)
///   - sibling: Merkle sibling for inclusion proof (simplified single-level)
///   - direction: 0 or 1 for Merkle path direction

template OwnershipProof() {
    // Public inputs
    signal input market_id;
    signal input resolved_outcome;
    signal input nullifier;
    signal input commitment_root;

    // Private inputs
    signal input secret_nonce;
    signal input amount;
    signal input original_pubkey;
    signal input sibling;
    signal input direction;

    // Step 1: Reconstruct commitment hash
    // commitment = Poseidon(market_id, resolved_outcome, amount, secret_nonce, original_pubkey)
    component commitHash = Poseidon(5);
    commitHash.inputs[0] <== market_id;
    commitHash.inputs[1] <== resolved_outcome;
    commitHash.inputs[2] <== amount;
    commitHash.inputs[3] <== secret_nonce;
    commitHash.inputs[4] <== original_pubkey;

    signal commitment;
    commitment <== commitHash.out;

    // Step 2: Verify Merkle inclusion (simplified single-level for PoC)
    // In production, this would be a full Merkle tree with multiple levels
    component merkleLeft = Poseidon(2);
    component merkleRight = Poseidon(2);

    // If direction == 0: hash(commitment, sibling)
    // If direction == 1: hash(sibling, commitment)
    merkleLeft.inputs[0] <== commitment;
    merkleLeft.inputs[1] <== sibling;

    merkleRight.inputs[0] <== sibling;
    merkleRight.inputs[1] <== commitment;

    component mux = Mux1();
    mux.c[0] <== merkleLeft.out;
    mux.c[1] <== merkleRight.out;
    mux.s <== direction;

    // Verify computed root matches public commitment_root
    mux.out === commitment_root;

    // Step 3: Verify nullifier correctness
    // nullifier = Poseidon(market_id, secret_nonce, original_pubkey)
    component nullHash = Poseidon(3);
    nullHash.inputs[0] <== market_id;
    nullHash.inputs[1] <== secret_nonce;
    nullHash.inputs[2] <== original_pubkey;

    nullHash.out === nullifier;

    // Step 4: Verify outcome is valid (0 or 1)
    resolved_outcome * (1 - resolved_outcome) === 0;
}

component main {public [market_id, resolved_outcome, nullifier, commitment_root]} = OwnershipProof();
