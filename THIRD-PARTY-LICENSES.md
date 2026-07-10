# Third-Party Licenses

`text-processing-rs` is licensed under the Apache License, Version 2.0 (see
`LICENSE` and `NOTICE`). Its distributions include or link the following
third-party works. All are permissive (Apache-2.0 / MIT) and compatible with
this crate's license.

---

## NVIDIA NeMo Text Processing

- **Used for:** the port's grammar logic, and — under the `fst-engine` feature —
  the compiled weighted-FST grammars in `grammars/` and the test fixtures in
  `tests/fixtures/`, which are derived from or copied from NeMo
  (pinned commit `1f1263579fe57ba7ed783cad3dddee710fcc5064`).
- **Source:** https://github.com/NVIDIA/NeMo-text-processing
- **License:** Apache License, Version 2.0
- **Copyright:** Copyright (c) NVIDIA CORPORATION & AFFILIATES.

---

## rustfst

- **Used for:** the `fst-engine` feature — loading and executing the compiled
  OpenFST grammars (composition + tropical shortest-path). Statically linked
  into `fst-engine` binary distributions.
- **Source:** https://github.com/Garvys/rustfst
- **License:** MIT OR Apache-2.0
- **Copyright:** Copyright (c) Alexandre Caulier and the rustfst contributors.

---

## flate2

- **Used for:** the `fst-engine` feature — decompressing the bundled gzipped
  grammars at load time. Statically linked into `fst-engine` binary
  distributions.
- **Source:** https://github.com/rust-lang/flate2-rs
- **License:** MIT OR Apache-2.0
- **Copyright:** Copyright (c) Alex Crichton and the flate2 contributors.

---

## Other Rust dependencies

`fst-engine` transitively links additional Rust crates via `rustfst` and
`flate2` (e.g. `nom`, `miniz_oxide`, `bitflags`, `anyhow`), and the default
build links `lazy_static`. All are distributed under permissive licenses
(MIT and/or Apache-2.0). See each crate's entry on <https://crates.io> and the
generated `Cargo.lock` for exact versions.

Full texts of the Apache License 2.0 and the MIT License are available at
<https://www.apache.org/licenses/LICENSE-2.0> and
<https://opensource.org/license/mit>.
