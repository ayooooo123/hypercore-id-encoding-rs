// node fixtures/gen/gen.js > fixtures/golden.json
const z32 = require('z32')
const id = require('hypercore-id-encoding')

// mulberry32: deterministic so the fixtures are reproducible
let seed = 7
function rand() {
  seed |= 0
  seed = (seed + 0x6d2b79f5) | 0
  let t = Math.imul(seed ^ (seed >>> 15), 1 | seed)
  t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t
  return ((t ^ (t >>> 14)) >>> 0) / 4294967296
}
const bytes = (n) => Buffer.from(Array.from({ length: n }, () => Math.floor(rand() * 256)))
const hex = (b) => Buffer.from(b).toString('hex')

const encode = []
for (let n = 0; n <= 40; n++) {
  const b = bytes(n)
  encode.push({ bytes: hex(b), z32: z32.encode(b) })
}
// Decoding any length, including ones encode never produces.
const decode = []
for (let n = 0; n <= 20; n++) {
  const s = Array.from({ length: n }, () => z32.ALPHABET[Math.floor(rand() * 32)]).join('')
  decode.push({ z32: s, bytes: hex(z32.decode(s)) })
}
const key = bytes(32)
const ids = []
for (const input of [id.encode(key), hex(key), 'pear://' + id.encode(key), 'pear://' + id.encode(key) + '/some/path', hex(key).toUpperCase(), 'short', id.encode(key).slice(1) + '0', hex(key).slice(2) + 'zz', '']) {
  ids.push({ input, key: id.isValid(input) ? hex(id.decode(input)) : null })
}
process.stdout.write(JSON.stringify({ encode, decode, ids, normalized: id.normalize(hex(key)) }, null, 1) + '\n')
