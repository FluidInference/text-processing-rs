# NeMo TN Parity: Status, Ceiling, and the Exact-Parity Route

This document records the English **Text Normalization (TN)** parity of this
crate against [NVIDIA NeMo-text-processing](https://github.com/NVIDIA/NeMo-text-processing),
the measured ceiling for a rule-based port, and a validated path to byte-exact
NeMo parity for anyone who needs it.

TL;DR:

- This crate (rule-based, pure Rust) is at **506/566 (89%)** on the en TN suite.
- **NeMo's own normalizer scores 480/508 (94.5%) on its own test files** — so
  **100% on these files is impossible for any single implementation**, NeMo
  included (the files aggregate several NeMo modes/configs).
- The optional **`fst-engine`** feature runs NeMo's *actual* compiled grammars
  via [`rustfst`](https://crates.io/crates/rustfst) and reaches **byte-exact
  NeMo parity for all seven languages** (zh, ja, fr, es, de, en, hi — 100% each
  vs NeMo's own deterministic output). Cost: the `rustfst`/`flate2` deps and
  ~7 MB of gzipped grammars. See "The FST route, shipped" below.

## How parity is measured

`examples/parity.rs` runs the port against NeMo's own upstream test fixtures
(`tests/nemo_text_processing/en/data_text_normalization/test_cases_*.txt`,
pinned commit), `catch_unwind`-isolated (panics = fails), compared in
sentence mode, and guarded against regressions by the committed
`tests/parity_baseline.tsv`. Run:

```bash
NEMO_DIR=/path/to/NeMo-text-processing cargo run --release --example parity
```

## Current en TN parity (this crate, rule-based)

| class | pass | class | pass | class | pass |
|-------|------|-------|------|-------|------|
| cardinal | 18/18 | fraction | 16/16 | roman | 4/4 |
| decimal | 12/12 | math | 4/4 | serial | 29/32 |
| date | 54/54 | measure | 14/21 | special_text | 9/10 |
| electronic | 43/45 | money | 71/71 | telephone | 20/20 |
| ordinal | 27/27 | range | 20/20 | time | 21/21 |
| whitelist | 7/7 | word | 39/39 | address | 8/11 |
| punctuation | 43/63 | punctuation_match_input | 7/13 | normalize_with_audio | 40/58 |

**Total: 506/566 (89%).** Thirteen classes are at 100%.

## Why 100% is not achievable (and never was)

The `test_cases_*.txt` files are **not the output of a single normalizer** —
they are a union of expected outputs across different NeMo **modes and
configurations**:

- `deterministic=True` (the default single-output grammar)
- non-deterministic mode (multiple transductions)
- **`punctuation_match_input`** — a separate mode that preserves the input's
  exact spacing (e.g. `23rd july,1998` → `...july,nineteen...`, *no* space),
  which directly conflicts with the spacing the default mode produces
- **`normalize_with_audio`** — a *multi-candidate* format (one input, several
  accepted spoken variants), not a single-output class

No single normalizer can satisfy all of these at once. We confirmed this
empirically by running **NeMo's own Python normalizer** against the same 508
cases:

```
NeMo (deterministic, Python) own score: 480/508 (94.5%)
```

NeMo fails to reproduce its own test files. Its failures are the exact cases
that resist a rule-based port too, e.g.:

| input | NeMo Python | test file wants |
|-------|-------------|-----------------|
| `&hi;&hi;` | `and hi;and hi;` | `and hi; and hi;` |
| `978-0` | `...eight-zero` | `...eight - zero` |
| `"text 7" text ".` | `...text ".` | `...text".` |

So the practical ceiling on these files is **~94–96%**, and a rule-based port
at 89% is close to that.

## The exact-parity route (validated, prototyped at 96.1%)

Byte-exact NeMo behavior *is* achievable in Rust, and we prototyped it
end-to-end. The pipeline is:

1. Compile NeMo's grammars once (offline, via `pynini`) and export the
   `ClassifyFst` (~7.7 MB, 147,319 states) and `VerbalizeFst` (~1.6 MB) plus
   the English `PostProcessingFst` (~0.3 MB) to OpenFST binary.
2. In Rust, load them with [`rustfst`](https://crates.io/crates/rustfst) (it
   reads OpenFST binary directly).
3. Port NeMo's `normalize.py` driver: compose the input, tropical shortest-path
   to the tagged form, recursive-descent token parse, per-token field
   permutation, verbalize, then apply the post-processing FST.

Running this against the same suite scored **488/508 (96.1%)** — at or above
NeMo's own default (94.5%).

### Fidelity notes (non-obvious, cost real debugging)

`rustfst` is a faithful *reimplementation* of OpenFST but is not bit-identical
on some weighted operations. Three fixes were required to match NeMo:

- **`rm_epsilon` before shortest-path** — composition epsilons otherwise break
  weight accumulation (e.g. dropped NeMo's British "and": `$18854` →
  `...eight hundred *and* fifty four`). Worth ~+9.5%.
- **A custom tropical DAG shortest-path** — `rustfst::shortest_path` diverges
  from OpenFST on complex compositions (isolated by round-tripping: pynini on
  rustfst's *own* composed FST gives the right answer, so it is the shortest
  path, not compose). The composed FST is acyclic (finite input), so a
  topological relaxation matches OpenFST. Worth ~+3.5%.
- **A quote-aware token parser** — a token value ends at a `"` *followed by a
  space*, so internal quotes are kept (`name: ""He's"` → value `"He's`).

### The tradeoff

Exact parity trades this crate's defining properties for byte-exactness:

| | rule-based (this crate) | rustfst + NeMo grammars |
|--|--|--|
| en TN parity | 89% | 96% (NeMo-parity) |
| bundle size | tiny, pure Rust | ~10 MB FSTs/lang + `rustfst` |
| dependencies | none | `rustfst` (OpenFST reimpl) |
| WASM | small | heavy |

For an on-device / embeddable use case, the rule-based port is the right shape.
The rustfst route is warranted only when byte-exact NeMo output is a hard
product requirement — and even then the ceiling is ~95% on these mixed-mode
test files, because (as shown above) NeMo itself cannot reach 100% on them.

## The FST route, shipped (`fst-engine` feature)

The FST engine is **language-agnostic** — the driver is unchanged per language;
only the compiled grammars swap, and each language sets a token-join separator
and whether it has a post-processing FST. Shipped behind the **`fst-engine`**
cargo feature (off by default; the rule-based path stays the default and carries
no new weight). **All seven languages reach byte-exact parity with NeMo:**

| lang | parity vs NeMo | separator | post-proc FST | rule-based baseline |
|------|----------------|-----------|---------------|---------------------|
| zh (Mandarin) | **367/367 (100%)** | `""` | — | ~7% |
| ja (Japanese) | **542/542 (100%)** | `""` | — | ~0% |
| fr (French) | **116/116 (100%)** | `" "` | — | ~60% |
| es (Spanish) | **536/536 (100%)** | `" "` | — | ~23% |
| de (German) | **314/314 (100%)** | `" "` | — | ~15% |
| en (English) | **506/506 (100%)** | `" "` | ✓ | ~89% |
| hi (Hindi) | **677/677 (100%)** | `" "` | ✓ | ~2% |

"Parity vs NeMo" means the engine reproduces `Normalizer(lang=…,
deterministic=True)` **byte-for-byte** — verified against NeMo's own output, not
just the published fixtures. This distinction matters: for zh/ja/fr/hi NeMo's own
normalizer reproduces its published fixtures exactly, but for en/es/de the
fixtures aggregate contradictory modes (see above), so the test oracle is NeMo's
*actual* deterministic output (`tests/fixtures/<lang>/` are regenerated from
NeMo; de's fixtures are also column-reversed spoken~written upstream).

```rust
// cargo build --features fst-engine
use text_processing_rs::fst::{zh, en, hi};
assert_eq!(zh::normalize("2024年"), "二零二四年");
assert_eq!(en::normalize("$2"), "two dollars");
```

### Reproducing NeMo's post-processing

The FST output is only half of NeMo's `normalize()`. After verbalizing and
joining tokens, NeMo runs a Python post-processing chain that the engine ports in
`driver.rs`:

1. **Collapse spaces + strip** (`SPACE_DUP`).
2. **Post-processing FST** — English/Hindi only (spacing/quote normalization
   baked into a grammar).
3. **Moses detokenization** — the engine reproduces the one behavior its outputs
   depend on: drop a space before `.,!?;:`.
4. **`post_process_punct`** — re-align punctuation spacing to the *original*
   input, so `978-0` reads `…eight-zero` (no spaces) while `9 - 0` keeps them.

Skipping steps 3–4 costs ~9% (en 91% → 100%); every one of those misses was a
whitespace difference, not a grammar difference.

- Grammars: `grammars/<lang>/*.fst.gz` — bundled **gzipped** (~7 MB for all seven
  languages combined; raw they are ~90 MB) and decompressed once at first use via
  `flate2`.
- Engine: `src/fst/` — `engine.rs` (the two fidelity fixes above), `driver.rs`
  (classify → parse → permute → verbalize → join → post-process), one small
  `<lang>.rs` per language (bundled grammars + separator + optional post-proc).
- Parity test: `tests/fst_parity.rs` runs the bundled NeMo goldens
  (`tests/fixtures/<lang>/`) and asserts 100% per language; the `fst-engine` CI
  job guards all seven without a NeMo checkout.
