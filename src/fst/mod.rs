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

/// A semantic class assigned by the NeMo TN classifier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Word,
    Punctuation,
    Cardinal,
    Ordinal,
    Decimal,
    Fraction,
    Time,
    Measure,
    Percent,
    Date,
    Telephone,
    Money,
    Electronic,
    Verbatim,
    Letters,
    Abbreviation,
    Other(String),
}

impl TokenKind {
    /// Stable lower-case name suitable for serialization and foreign-language
    /// bindings.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Word => "word",
            Self::Punctuation => "punctuation",
            Self::Cardinal => "cardinal",
            Self::Ordinal => "ordinal",
            Self::Decimal => "decimal",
            Self::Fraction => "fraction",
            Self::Time => "time",
            Self::Measure => "measure",
            Self::Percent => "percent",
            Self::Date => "date",
            Self::Telephone => "telephone",
            Self::Money => "money",
            Self::Electronic => "electronic",
            Self::Verbatim => "verbatim",
            Self::Letters => "letters",
            Self::Abbreviation => "abbreviation",
            Self::Other(name) => name,
        }
    }
}

/// One source span and the words produced for it by text normalization.
///
/// Offsets are half-open UTF-8 byte offsets into the original input. Whitespace
/// between spans is deliberately excluded, so callers can recover it from the
/// gaps between adjacent ranges without losing the original formatting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlignedSpan {
    pub input_start: usize,
    pub input_end: usize,
    pub original: String,
    /// Direct verbalizer output for this classifier token. Sentence-level
    /// punctuation and spacing cleanup is reflected in
    /// [`AlignedNormalization::normalized`].
    pub normalized: String,
    pub kind: TokenKind,
}

/// Sentence-level TN output together with its source-to-normalized mapping.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlignedNormalization {
    /// The same sentence-level result returned by the corresponding
    /// language's [`normalize`](crate::fst::en::normalize) function.
    pub normalized: String,
    pub spans: Vec<AlignedSpan>,
}

/// Normalize with NeMo's compiled FST grammar and preserve source spans.
///
/// Supported language codes are `en`, `fr`, `es`, `de`, `zh`, `hi`, and `ja`.
/// Returns `None` for unsupported languages or when classification fails.
pub fn normalize_aligned(input: &str, lang: &str) -> Option<AlignedNormalization> {
    match lang {
        "en" => en::normalize_aligned(input),
        "fr" => fr::normalize_aligned(input),
        "es" => es::normalize_aligned(input),
        "de" => de::normalize_aligned(input),
        "zh" => zh::normalize_aligned(input),
        "hi" => hi::normalize_aligned(input),
        "ja" => ja::normalize_aligned(input),
        _ => None,
    }
}

/// Decompress a bundled `*.fst.gz` grammar and load it as an FST.
fn load_gz(gz: &[u8]) -> VectorFst<TropicalWeight> {
    let mut bytes = Vec::new();
    GzDecoder::new(gz)
        .read_to_end(&mut bytes)
        .expect("bundled grammar is valid gzip");
    VectorFst::load(&bytes).expect("bundled grammar is valid OpenFST binary")
}
