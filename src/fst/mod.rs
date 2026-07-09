//! Byte-exact NeMo parity engine (feature `fst-engine`).
//!
//! Where the rule-based taggers asymptote below NeMo (a priority matcher can't
//! reproduce a weighted-FST's shortest-path disambiguation), this engine runs
//! NeMo's *actual* compiled grammars via [`rustfst`]. The pipeline mirrors
//! NeMo's `normalize.py`: compose the input against the classifier, take the
//! tropical shortest path to a tagged form, parse and permute its fields,
//! verbalize each token, and join.
//!
//! This trades the crate's pure-Rust, tiny-bundle shape for byte-exactness and
//! is therefore optional and off by default. It is warranted for languages
//! where hand-written rules dead-end (e.g. Mandarin number reading) or where
//! byte-exact NeMo output is a hard requirement.
//!
//! See `docs/NEMO_PARITY.md` for the measured ceilings and the fidelity fixes.

mod driver;
mod engine;

pub mod fr;
pub mod zh;
