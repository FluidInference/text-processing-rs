# text-processing-rs

A Rust port of [NVIDIA NeMo Text Processing](https://github.com/NVIDIA/NeMo-text-processing) for Inverse Text Normalization (ITN).

## What it does

Converts spoken-form ASR output to written form:

| Input | Output |
|-------|--------|
| two hundred thirty two | 232 |
| five dollars and fifty cents | $5.50 |
| january fifth twenty twenty five | January 5, 2025 |
| quarter past two pm | 02:15 p.m. |
| one point five billion dollars | $1.5 billion |
| seventy two degrees fahrenheit | 72 °F |

## Usage

### Rust

```rust
use nemo_text_processing::normalize;

let result = normalize("two hundred");
assert_eq!(result, "200");

let result = normalize("five dollars and fifty cents");
assert_eq!(result, "$5.50");
```

### Swift

```swift
import NemoTextProcessing

let result = NemoTextProcessing.normalize("two hundred")
// result is "200"

let money = NemoTextProcessing.normalize("five dollars and fifty cents")
// money is "$5.50"
```

## Compatibility

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

## Features

- Cardinal and ordinal number conversion
- Decimal numbers with scale words (million, billion)
- Currency formatting (USD, with scale words)
- Measurements including temperature (°C, °F, K) and data rates (gbps)
- Date parsing (multiple formats)
- Time parsing with AM/PM and timezone preservation
- Email and URL normalization
- Phone numbers, IP addresses, SSN
- Case preservation for proper nouns and abbreviations

## Building

### Rust

```bash
cargo build
cargo test
```

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

Apache 2.0 (same as [NeMo Text Processing](https://github.com/NVIDIA/NeMo-text-processing))

## Acknowledgments

This project is a Rust implementation based on the inverse text normalization grammars from [NVIDIA NeMo Text Processing](https://github.com/NVIDIA/NeMo-text-processing). All credit for the original algorithms and test cases goes to the NVIDIA NeMo team.
