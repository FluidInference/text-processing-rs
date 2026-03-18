import fs from 'node:fs';
import path from 'node:path';

const pkgDir = process.argv[2];
if (!pkgDir) {
  throw new Error('Usage: node scripts/set-wasm-package-name.mjs <pkg-dir>');
}

const packageJsonPath = path.join(pkgDir, 'package.json');
const pkg = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
pkg.name = '@fluidinference/text-processing-rs';
pkg.keywords = ['asr', 'speech', 'normalization', 'nlp', 'itn', 'tts', 'wasm'];
fs.writeFileSync(packageJsonPath, `${JSON.stringify(pkg, null, 2)}\n`);
