const BN = require("bn.js");
const proof = require("./build/proof.json");
const publicInputs = require("./build/public.json");

// BN254 field prime
const P = new BN("21888242871839275222246405745257275088696311157297823662689037894645226208583");

function toBytes32BE(decStr) {
  return Array.from(new BN(decStr).toArrayLike(Buffer, "be", 32));
}

function negateG1Y(yStr) {
  const y = new BN(yStr);
  return P.sub(y).toArrayLike(Buffer, "be", 32);
}

// proof_a: negate y-coordinate (required by groth16-solana)
const proof_a = [
  ...toBytes32BE(proof.pi_a[0]),
  ...Array.from(negateG1Y(proof.pi_a[1])),
];

// proof_b: swap x coordinates (snarkjs uses [x1,x2],[y1,y2] but solana wants [x2,x1],[y2,y1])
const proof_b = [
  ...toBytes32BE(proof.pi_b[0][1]),
  ...toBytes32BE(proof.pi_b[0][0]),
  ...toBytes32BE(proof.pi_b[1][1]),
  ...toBytes32BE(proof.pi_b[1][0]),
];

// proof_c: as-is
const proof_c = [
  ...toBytes32BE(proof.pi_c[0]),
  ...toBytes32BE(proof.pi_c[1]),
];

// public inputs as [u8;32] each
const pubInputs = publicInputs.map(toBytes32BE);

console.log("// Copy these into your TypeScript test\n");
console.log("const proofA = new Uint8Array(" + JSON.stringify(proof_a) + ");\n");
console.log("const proofB = new Uint8Array(" + JSON.stringify(proof_b) + ");\n");
console.log("const proofC = new Uint8Array(" + JSON.stringify(proof_c) + ");\n");
console.log("const publicInput0_marketId = new Uint8Array(" + JSON.stringify(pubInputs[0]) + ");");
console.log("const publicInput1_outcome = new Uint8Array(" + JSON.stringify(pubInputs[1]) + ");");
console.log("const publicInput2_nullifier = new Uint8Array(" + JSON.stringify(pubInputs[2]) + ");");
console.log("const publicInput3_commitmentRoot = new Uint8Array(" + JSON.stringify(pubInputs[3]) + ");");
