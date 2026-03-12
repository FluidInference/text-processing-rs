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
use nemo_text_processing::{normalize, tn_normalize};

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

Sentence-level normalization scans for normalizable spans within a larger sentence:

```rust
use nemo_text_processing::{normalize_sentence, tn_normalize_sentence};

// ITN sentence mode
let result = normalize_sentence("I have twenty one apples");
assert_eq!(result, "I have 21 apples");

// TN sentence mode
let result = tn_normalize_sentence("I paid $5 for 23 items");
assert_eq!(result, "I paid five dollars for twenty three items");
```

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

let tn = NemoTextProcessing.tnNormalizeSentence("I paid $5 for 23 items")
// "I paid five dollars for twenty three items"
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
- Custom rules for domain-specific terms
- C FFI for integration with Swift, Python, and other languages

## Building

### Rust

```bash
cargo build
cargo test
```

### CLI Tools

```bash
# Build the Rust library (release, with FFI)
cargo build --release --target aarch64-apple-darwin --features ffi

# Build Swift CLI tools
cd swift-test && swift build
```

Binaries are at `swift-test/.build/debug/nemo-itn` and `swift-test/.build/debug/nemo-tn`.

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
