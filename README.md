# text-processing-rs

A Rust port of [NVIDIA NeMo Text Processing](https://github.com/NVIDIA/NeMo-text-processing) supporting both **Inverse Text Normalization (ITN)** and **Text Normalization (TN)**.

## What it does

### ITN: Spoken → Written

Converts spoken-form ASR output to written form:

| Input | Output |
|-------|--------|
| two hundred thirty two | 232 |
| five dollars and fifty cents | $5.50 |
| january fifth twenty twenty five | January 5, 2025 |
| quarter past two pm | 02:15 p.m. |
| one point five billion dollars | $1.5 billion |
| seventy two degrees fahrenheit | 72 °F |

### TN: Written → Spoken

Converts written-form text to spoken form (useful for TTS preprocessing):

| Input | Output |
|-------|--------|
| 123 | one hundred twenty three |
| $5.50 | five dollars and fifty cents |
| January 5, 2025 | january fifth twenty twenty five |
| 2:30 PM | two thirty p m |
| 1st | first |
| 200 km/h | two hundred kilometers per hour |

## Usage

### Rust

```rust
use text_processing_rs::{normalize, tn_normalize};

// ITN: spoken → written
let result = normalize("two hundred");
assert_eq!(result, "200");

let result = normalize("five dollars and fifty cents");
assert_eq!(result, "$5.50");

// TN: written → spoken
let result = tn_normalize("$5.50");
assert_eq!(result, "five dollars and fifty cents");

let result = tn_normalize("123");
assert_eq!(result, "one hundred twenty three");
```

### JavaScript (WASM)

Build wasm artifacts:

```bash
npm run wasm:build:node
npm run wasm:build:web
```

Node usage:

```javascript
import * as wasm from "./pkg-node/text_processing_rs.js";

console.log(wasm.normalize("two hundred")); // "200"
console.log(wasm.tnNormalize("$5.50")); // "five dollars and fifty cents"

wasm.addRule("gee pee tee", "GPT");
console.log(wasm.normalize("gee pee tee")); // "GPT"
```

The generated npm package name is `@fluidinference/text-processing-rs`.

Web project usage (Vite / Next.js / webpack):

```bash
npm install @fluidinference/text-processing-rs
```

```javascript
import init, * as wasm from "@fluidinference/text-processing-rs";

async function run() {
  // Loads and initializes the .wasm module (required once at startup)
  await init();

  const itn = wasm.normalize("two hundred");
  const tn = wasm.tnNormalize("$5.50");

  console.log(itn); // "200"
  console.log(tn); // "five dollars and fifty cents"

  wasm.addRule("gee pee tee", "GPT");
  console.log(wasm.normalize("gee pee tee")); // "GPT"
}

run();
```

If your framework supports top-level `await`, you can initialize at module load time:

```javascript
import init, * as wasm from "@fluidinference/text-processing-rs";
await init();
```

Sentence-level normalization scans for normalizable spans within a larger sentence:

```rust
use text_processing_rs::{normalize_sentence, normalize_sentence_lang, tn_normalize_sentence};

// ITN sentence mode
let result = normalize_sentence("I have twenty one apples");
assert_eq!(result, "I have 21 apples");

// ITN sentence mode, language-aware ("en", "fr", "es", "de", "zh", "hi", "ja")
let result = normalize_sentence_lang("j'ai vingt et un ans", "fr");
assert_eq!(result, "j'ai 21 ans");

// TN sentence mode
let result = tn_normalize_sentence("I paid $5 for 23 items");
assert_eq!(result, "I paid five dollars for twenty three items");
```

Compiled-FST TN can also retain the source range and semantic class for every
normalized span. Offsets are half-open UTF-8 byte offsets:

```rust
use text_processing_rs::fst;

let result = fst::normalize_aligned("The price is $1,234.56.", "en").unwrap();
assert_eq!(
    result.normalized,
    "The price is one thousand two hundred and thirty four dollars fifty six cents."
);

let money = &result.spans[3];
assert_eq!(money.input_start, 13);
assert_eq!(money.input_end, 22);
assert_eq!(money.original, "$1,234.56");
assert_eq!(money.kind.as_str(), "money");
```

Build this API with `--features fst-engine`. It uses the same compiled NeMo
classifier and verbalizer as `fst::<lang>::normalize`, rather than recovering
alignment by diffing the final strings.

### Swift

```swift
import NemoTextProcessing

// ITN: spoken → written
let result = NemoTextProcessing.normalize("two hundred")
// "200"

// TN: written → spoken
let spoken = NemoTextProcessing.tnNormalize("$5.50")
// "five dollars and fifty cents"

// Sentence modes
let itn = NemoTextProcessing.normalizeSentence("I have twenty one apples")
// "I have 21 apples"

// Language-aware ITN sentence mode ("en", "fr", "es", "de", "zh", "hi", "ja")
let itnFr = NemoTextProcessing.normalizeSentence("j'ai vingt et un ans", language: "fr")
// "j'ai 21 ans"

let tn = NemoTextProcessing.tnNormalizeSentence("I paid $5 for 23 items")
// "I paid five dollars for twenty three items"

if let aligned = NemoTextProcessing.tnNormalizeAligned(
    "The price is $1,234.56.",
    language: "en"
) {
    let money = aligned.spans[3]
    // money.original == "$1,234.56"
    // money.normalized == "one thousand ... dollars fifty six cents"
    // money.inputRange == 13..<22, money.kind == "money"
}
```

### CLI

```bash
# ITN
nemo-itn two hundred thirty two        # → 232
nemo-itn -s "I have twenty one apples" # → I have 21 apples

# TN
nemo-tn 123                            # → one hundred twenty three
nemo-tn '$5.50'                        # → five dollars and fifty cents
nemo-tn -s 'I paid $5 for 23 items'    # → I paid five dollars for twenty three items

# Pipe from stdin
echo "2:30 PM" | nemo-tn               # → two thirty p m
```

## Compatibility

### ITN (Spoken → Written)

**98.6% compatible** with NeMo text processing test suite (1200/1217 tests passing).

| Category | Status |
|----------|--------|
| Cardinal numbers | 100% |
| Ordinal numbers | 100% |
| Decimal numbers | 100% |
| Money | 100% |
| Measurements | 100% |
| Dates | 100% |
| Time | 97% |
| Electronic (email/URL) | 96% |
| Telephone/IP | 96% |
| Whitelist terms | 100% |

### TN (Written → Spoken)

| Category | Examples |
|----------|----------|
| Cardinal numbers | `123` → `one hundred twenty three` |
| Ordinal numbers | `1st` → `first`, `21st` → `twenty first` |
| Decimal numbers | `3.14` → `three point one four` |
| Money | `$5.50` → `five dollars and fifty cents` |
| Measurements | `200 km/h` → `two hundred kilometers per hour` |
| Dates | `January 5, 2025` → `january fifth twenty twenty five` |
| Time | `2:30 PM` → `two thirty p m` |
| Electronic (email/URL) | `test@gmail.com` → `t e s t at g m a i l dot c o m` |
| Telephone | `123-456-7890` → `one two three, four five six, seven eight nine zero` |
| Whitelist terms | `Dr.` → `doctor`, `Mr.` → `mister` |

## Features

- **ITN** (Inverse Text Normalization): spoken → written form for ASR post-processing
- **TN** (Text Normalization): written → spoken form for TTS preprocessing
- Cardinal and ordinal number conversion (both directions)
- Decimal numbers with scale words (million, billion)
- Currency formatting (USD, GBP, EUR, JPY, and more)
- Measurements including temperature (°C, °F, K) and data rates (gbps)
- Date parsing (multiple formats) and decade verbalization (1980s → nineteen eighties)
- Time parsing with AM/PM, 24-hour format, and timezone preservation
- Email and URL normalization
- Phone numbers, IP addresses, SSN
- Case preservation for proper nouns and abbreviations
- Sentence-level normalization with sliding window span matching
- Source-to-normalized span alignment with semantic classes (compiled FST)
- Custom rules for domain-specific terms
- C FFI for integration with Swift, Python, and other languages

## Building

### Rust

```bash
cargo build
cargo test
```

### WASM + JavaScript

```bash
# Build + smoke test (Node) + build browser artifact
npm run wasm:ci

# Create a tarball from the browser package
npm run wasm:pack

# Publish browser package to npm (requires npm auth)
npm run wasm:publish
```

### CLI Tools

```bash
# Build the Rust library for this Mac's architecture.
RUST_TARGET="$(rustc -vV | sed -n 's/^host: //p')"
cargo build --release --target "$RUST_TARGET" --features "ffi,fst-engine"

# Build Swift CLI tools
cd swift-test && swift build
```

Binaries are at `swift-test/.build/debug/nemo-itn`,
`swift-test/.build/debug/nemo-tn`, and
`swift-test/.build/debug/nemo-tn-aligned`.

#### nemo-tn
```bash
swift-test/.build/debug/nemo-tn -s 'The price is $1,234.56.'
# output: The price is one thousand two hundred and thirty four point five six dollars
```

#### nemo-tn-aligned
The aligned CLI emits compact JSON for argument input and JSON Lines for stdin:

```bash
swift-test/.build/debug/nemo-tn-aligned --lang en 'The price is $1,234.56.'
# output: {"input":"The price is $1,234.56.","language":"en","normalized":"The price is one thousand two hundred and thirty four dollars fifty six cents.","spans":[{"input_end":3,"input_start":0,"kind":"word","normalized":"The","original":"The"},{"input_end":9,"input_start":4,"kind":"word","normalized":"price","original":"price"},{"input_end":12,"input_start":10,"kind":"word","normalized":"is","original":"is"},{"input_end":22,"input_start":13,"kind":"money","normalized":"one thousand two hundred and thirty four dollars fifty six cents","original":"$1,234.56"},{"input_end":23,"input_start":22,"kind":"punctuation","normalized":".","original":"."}]}
```

Each result contains `input`, `language`, the complete `normalized` text, and
`spans` with `input_start`, `input_end`, `original`, `normalized`, and `kind`.
Offsets are half-open UTF-8 byte offsets.

### Swift (XCFramework)

```bash
# Install Rust targets
rustup target add aarch64-apple-darwin x86_64-apple-darwin
rustup target add aarch64-apple-ios aarch64-apple-ios-sim

# Build XCFramework
./build-xcframework.sh
```

Output:
- `output/NemoTextProcessing.xcframework` - Add to Xcode project
- `output/NemoTextProcessing.swift` - Swift wrapper

## License

Apache 2.0

## Acknowledgments

This project is a Rust implementation based on the inverse text normalization grammars from [NVIDIA NeMo Text Processing](https://github.com/NVIDIA/NeMo-text-processing). All credit for the original algorithms and test cases goes to the NVIDIA NeMo team.
