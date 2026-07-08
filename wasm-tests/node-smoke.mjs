import * as wasm from '../pkg-node/text_processing_rs.js';

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

assertEqual(wasm.normalize('two hundred'), '200', 'normalize should convert spoken numbers');
assertEqual(
  wasm.normalizeWithLang('two hundred', 'en'),
  '200',
  'normalizeWithLang should work'
);
assertEqual(
  wasm.normalizeSentence('I have twenty one apples'),
  'I have 21 apples',
  'normalizeSentence should convert spans'
);
assertEqual(wasm.tnNormalize('$5.50'), 'five dollars fifty cents', 'tnNormalize should work');
assertEqual(
  wasm.tnNormalizeSentence('I paid $5 for 23 items'),
  'I paid five dollars for twenty three items',
  'tnNormalizeSentence should convert spans'
);

wasm.clearRules();
assertEqual(wasm.ruleCount(), 0, 'ruleCount starts at 0');
wasm.addRule('gee pee tee', 'GPT');
assertEqual(wasm.ruleCount(), 1, 'ruleCount increments');
assertEqual(wasm.normalize('gee pee tee'), 'GPT', 'custom rules should apply');
assertTrue(wasm.removeRule('gee pee tee'), 'removeRule should return true when found');
assertEqual(wasm.ruleCount(), 0, 'rule removed');
assertTrue(!wasm.removeRule('gee pee tee'), 'removeRule should return false when missing');

console.log('WASM node smoke test passed');
