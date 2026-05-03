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
///   - commitment_root: Merkle root of all commitments
///   - amount: bet amount (public to prevent commitment decoupling)
///
/// Private inputs (witnesses):
///   - secret_nonce: random nonce used when placing bet
///   - original_pubkey: pubkey used when placing bet (field element)
///   - sibling[10]: Merkle siblings for inclusion proof (depth-10 tree)
///   - direction[10]: 0 or 1 for Merkle path direction at each level

template OwnershipProof() {
    // Public inputs
    signal input market_id;
    signal input resolved_outcome;
    signal input nullifier;
    signal input commitment_root;
    signal input amount;

    // Private inputs
    signal input secret_nonce;
    signal input original_pubkey;
    signal input sibling[10];
    signal input direction[10];

    // Step 1: Reconstruct commitment hash
    // commitment = Poseidon(market_id, resolved_outcome, amount, secret_nonce, original_pubkey)
    component commitHash = Poseidon(5);
    commitHash.inputs[0] <== market_id;
    commitHash.inputs[1] <== resolved_outcome;
    commitHash.inputs[2] <== amount;
    commitHash.inputs[3] <== secret_nonce;
    commitHash.inputs[4] <== original_pubkey;

    // Step 2: Verify Merkle inclusion (depth-10 incremental Merkle tree)
    // Matches on-chain Poseidon Merkle tree in place_bet.rs
    component merkleLeft[10];
    component merkleRight[10];
    component mux[10];

    signal levelHash[11];
    levelHash[0] <== commitHash.out;

    for (var i = 0; i < 10; i++) {
        // Constrain direction to be boolean (0 or 1) to prevent forgery
        direction[i] * (1 - direction[i]) === 0;

        merkleLeft[i] = Poseidon(2);
        merkleRight[i] = Poseidon(2);

        // If direction == 0: hash(current, sibling)
        // If direction == 1: hash(sibling, current)
        merkleLeft[i].inputs[0] <== levelHash[i];
        merkleLeft[i].inputs[1] <== sibling[i];

        merkleRight[i].inputs[0] <== sibling[i];
        merkleRight[i].inputs[1] <== levelHash[i];

        mux[i] = Mux1();
        mux[i].c[0] <== merkleLeft[i].out;
        mux[i].c[1] <== merkleRight[i].out;
        mux[i].s <== direction[i];

        levelHash[i + 1] <== mux[i].out;
    }

    // Verify computed root matches public commitment_root
    levelHash[10] === commitment_root;

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

component main {public [market_id, resolved_outcome, nullifier, commitment_root, amount]} = OwnershipProof();
