//! Byte-exact NeMo parity engine (feature `fst-engine`).
//!
//! Where the rule-based taggers asymptote below NeMo (a priority matcher can't
//! reproduce a weighted-FST's shortest-path disambiguation), this engine runs
//! NeMo's *actual* compiled grammars via [`rustfst`]. The pipeline mirrors
//! NeMo's `normalize.py`: compose the input against the classifier, take the
//! tropical shortest path to a tagged form, parse and permute its fields,
//! verbalize each token, join, and reproduce NeMo's post-processing
//! (space collapse, optional post-processing FST, Moses detokenization,
//! punctuation re-alignment to the original input).
//!
//! Every language reaches byte-exact parity with
//! `Normalizer(lang=…, deterministic=True)`:
//! zh 367/367, fr 116/116, ja 542/542, en 506/506, hi 677/677, es 536/536,
//! de 314/314.
//!
//! This trades the crate's pure-Rust, tiny-bundle shape for byte-exactness and
//! is therefore optional and off by default. Grammars are bundled gzipped
//! (~7 MB total for all languages) and decompressed once at first use.
//!
//! See `docs/NEMO_PARITY.md` for the measured ceilings and the fidelity fixes.

mod driver;
mod engine;

pub mod de;
pub mod en;
pub mod es;
pub mod fr;
pub mod hi;
pub mod ja;
pub mod zh;

use flate2::read::GzDecoder;
use rustfst::prelude::*;
use std::io::Read;

/// Decompress a bundled `*.fst.gz` grammar and load it as an FST.
fn load_gz(gz: &[u8]) -> VectorFst<TropicalWeight> {
    let mut bytes = Vec::new();
    GzDecoder::new(gz)
        .read_to_end(&mut bytes)
        .expect("bundled grammar is valid gzip");
    VectorFst::load(&bytes).expect("bundled grammar is valid OpenFST binary")
}
