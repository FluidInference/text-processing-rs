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

- `src/asr/` - ITN taggers (spoken to written, for ASR/STT post-processing)
- `src/tts/` - TN taggers (written to spoken, for TTS preprocessing)
- `src/custom_rules.rs` - User-defined custom normalization rules
- `src/ffi.rs` - C FFI bindings for Swift/Python integration
- `tests/` - Integration and extensive edge-case tests
