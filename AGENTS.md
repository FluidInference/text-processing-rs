# text-processing-rs - Agent Development Guide

## Build & Test Commands

```bash
cargo build                      # Build project
cargo build --release            # Release build
cargo build --features ffi       # Build with C FFI bindings
cargo build --features wasm --target wasm32-unknown-unknown  # Build for WebAssembly
cargo test                       # Run all tests
cargo test --test en_tests       # Run a single integration test file
cargo test en_tn_tests::test_name  # Run a single test by name
cargo fmt                        # Format code
cargo clippy --all-targets       # Lint
```

## Architecture

- **src/itn/**: ITN taggers (spoken → written, Inverse Text Normalization) per language (en, de, es, fr, hi, ja, zh)
- **src/tn/**: TN taggers (written → spoken, Text Normalization)
- **src/custom_rules.rs**: User-defined custom normalization rules (highest priority)
- **src/ffi.rs**: C FFI bindings for Swift/Python integration (gated by `ffi` feature)
- **src/wasm.rs**: JavaScript-callable wasm bindings (gated by `wasm` feature)
- **src/lib.rs**: Public API entry point — `normalize()` dispatches to taggers in specificity order
- **tests/**: Integration tests (per-language ITN/TN suites + `extensive_tests.rs` edge cases)
- **swift/**, **swift-test/**: Swift consumer package and tests for the FFI surface
- **wasm-tests/**: Browser/Node tests against the wasm build
- **scripts/**: Build helpers (e.g. `set-wasm-package-name.mjs`)
- **build-xcframework.sh**: Builds an `.xcframework` for Apple platform consumers

### Processing Pipeline

```
input → custom_rules → whitelist → punctuation → word → time → telephone → ... → output
```

Taggers are tried in order of specificity (most specific first). If no tagger matches, the original text is returned.

## Critical Rules

- **NEVER** run `git push` unless explicitly requested by the user
- **NEVER** add `Co-Authored-By` lines for Claude, Copilot, or any AI assistant in commit messages
- **NEVER** create simplified or stub versions — implement full solutions or consult first
- **NEVER** introduce mock data or fabricated test cases — use realistic spoken/written forms
- Add unit/integration tests when adding new taggers or rules
- Keep all changes local to the appropriate language module under `src/itn/<lang>/` or `src/tn/<lang>/`
- Maintain feature-flag isolation: code behind `ffi` and `wasm` must not leak into the default build

## Code Style (rustfmt + clippy)

- Use `cargo fmt` before committing — defaults to standard rustfmt rules
- Use `snake_case` for functions/variables, `UpperCamelCase` for types, `SCREAMING_SNAKE_CASE` for consts
- Public APIs: document with `///` doc comments and include runnable examples where practical
- Error handling: prefer `Option`/`Result`; avoid `unwrap()`/`expect()` in library code
- Control flow: prefer early returns and flat structure over nested conditionals
- Imports: group `std`, external crates, and local modules separately
- Keep tagger functions small and focused — one tagger per spoken-form pattern

## Clean Code

- Maintain API consistency across language taggers (each tagger exposes `parse(&str) -> Option<String>`)
- Single responsibility per file: one tagger category per module (e.g. `cardinal.rs`, `date.rs`, `money.rs`)
- New languages should mirror the existing `itn::en` module layout
- When extending the public API, update `lib.rs`, FFI bindings (`ffi.rs`), and wasm bindings (`wasm.rs`) together so all consumers stay in sync

## FFI / Bindings

- C FFI lives in `src/ffi.rs` behind the `ffi` feature; consumed by the Swift package in `swift/`
- Wasm bindings live in `src/wasm.rs` behind the `wasm` feature; consumed by `wasm-tests/`
- Run `cargo build --features ffi` and the Swift tests under `swift-test/` after touching FFI signatures
- Run the wasm test suite after touching `wasm.rs` or anything affecting the wasm-exposed surface

## Mobius Plan

Consult `PLANS.md` (when present) for complex tasks; plan changes first. Store plans in a local `.mobius/` folder without committing to GitHub.
