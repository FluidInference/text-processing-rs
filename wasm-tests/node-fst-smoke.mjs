import * as wasm from '../pkg-node-fst/text_processing_rs.js';

function assertEqual(actual, expected, message) {
  if (actual !== expected) {
    throw new Error(`${message}: expected "${expected}", got "${actual}"`);
  }
}

function assertTrue(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

const input = 'The price is $1,234.56.';
const result = wasm.tnFstNormalizeAligned(input, 'en');
assertTrue(result !== null, 'tnFstNormalizeAligned should align English input');
assertEqual(
  result.normalized,
  'The price is one thousand two hundred and thirty four dollars fifty six cents.',
  'tnFstNormalizeAligned should return the sentence-level result'
);
assertEqual(result.spans.length, 5, 'tnFstNormalizeAligned should return one span per token');

const money = result.spans[3];
assertEqual(money.inputStart, 13, 'money span start');
assertEqual(money.inputEnd, 22, 'money span end');
assertEqual(money.original, '$1,234.56', 'money span original');
assertEqual(
  money.normalized,
  'one thousand two hundred and thirty four dollars fifty six cents',
  'money span normalized'
);
assertEqual(money.kind, 'money', 'money span kind');

// Offsets are UTF-8 bytes, not UTF-16 code units.
const utf8 = new TextEncoder().encode('Café costs €12.50.');
const cafe = wasm.tnFstNormalizeAligned('Café costs €12.50.', 'en');
const euro = cafe.spans[2];
assertEqual(
  new TextDecoder().decode(utf8.slice(euro.inputStart, euro.inputEnd)),
  '€12.50',
  'offsets should slice the UTF-8 bytes of the input'
);

assertEqual(wasm.tnFstNormalizeAligned('$12.50', 'xx'), null, 'unsupported languages return null');

console.log('WASM fst-engine node smoke test passed');
