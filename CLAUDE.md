# CLAUDE.md

## Git Operations

- Never run `git push` unless explicitly requested
- No Co-Author tags: Do not add `Co-Authored-By` lines for Claude, Copilot, or any AI assistant in commit messages

## Build Commands

```bash
cargo build
cargo test
cargo fmt
cargo build --features ffi
```

## Project Structure

- `src/itn/` - ITN taggers (spoken to written, Inverse Text Normalization)
- `src/tn/` - TN taggers (written to spoken, Text Normalization)
- `src/custom_rules.rs` - User-defined custom normalization rules
- `src/ffi.rs` - C FFI bindings for Swift/Python integration
- `tests/` - Integration and extensive edge-case tests
