const fs = require('fs');

const vk = JSON.parse(fs.readFileSync('build/verification_key.json', 'utf8'));

// Convert string array [x, y, z] to big endian bytes
function g1ToBytes(point) {
  const x = BigInt(point[0]);
  const y = BigInt(point[1]);
  const xBytes = bigintToBytes32BE(x);
  const yBytes = bigintToBytes32BE(y);
  return [...xBytes, ...yBytes];
}

function g2ToBytes(point) {
  // G2 points: [[x0,x1],[y0,y1]]
  const x0 = BigInt(point[0][0]);
  const x1 = BigInt(point[0][1]);
  const y0 = BigInt(point[1][0]);
  const y1 = BigInt(point[1][1]);
  // Order in groth16-solana: x1,x0,y1,y0 (each 32 bytes BE)
  return [...bigintToBytes32BE(x1), ...bigintToBytes32BE(x0),
          ...bigintToBytes32BE(y1), ...bigintToBytes32BE(y0)];
}

function bigintToBytes32BE(n) {
  const hex = n.toString(16).padStart(64, '0');
  const bytes = [];
  for (let i = 0; i < 64; i += 2) {
    bytes.push(parseInt(hex.substr(i, 2), 16));
  }
  return bytes;
}

function formatRustArray(bytes, name) {
  let s = `pub const ${name}: [u8; ${bytes.length}] = [\n`;
  for (let i = 0; i < bytes.length; i += 16) {
    const chunk = bytes.slice(i, Math.min(i + 16, bytes.length));
    s += '    ' + chunk.join(', ') + ',\n';
  }
  s += '];\n';
  return s;
}

function formatRustIcArray(ics) {
  let s = `pub const VK_IC: [[u8; 64]; ${ics.length}] = [\n`;
  for (const ic of ics) {
    const bytes = g1ToBytes(ic);
    s += '    [\n';
    for (let i = 0; i < bytes.length; i += 16) {
      const chunk = bytes.slice(i, Math.min(i + 16, bytes.length));
      s += '        ' + chunk.join(', ') + ',\n';
    }
    s += '    ],\n';
  }
  s += '];\n';
  return s;
}

const alpha = g1ToBytes(vk.vk_alpha_1);
const beta = g2ToBytes(vk.vk_beta_2);
const gamma = g2ToBytes(vk.vk_gamma_2);
const delta = g2ToBytes(vk.vk_delta_2);

let output = '// Auto-generated from verification_key.json\n';
output += '// Circuit: ownership.circom (4 public inputs)\n\n';
output += formatRustArray(alpha, 'VK_ALPHA_G1');
output += '\n';
output += formatRustArray(beta, 'VK_BETA_G2');
output += '\n';
output += formatRustArray(gamma, 'VK_GAMMA_G2');
output += '\n';
output += formatRustArray(delta, 'VK_DELTA_G2');
output += '\n';
output += formatRustIcArray(vk.IC);

fs.writeFileSync('build/vk_bytes.rs', output);
console.log('Verifying key exported to build/vk_bytes.rs');
console.log('Alpha G1:', alpha.length, 'bytes');
console.log('Beta G2:', beta.length, 'bytes');
console.log('Gamma G2:', gamma.length, 'bytes');
console.log('Delta G2:', delta.length, 'bytes');
console.log('IC:', vk.IC.length, 'entries x 64 bytes');
