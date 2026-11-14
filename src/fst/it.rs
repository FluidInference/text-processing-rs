//! Italian (it) text normalization via NeMo's compiled grammars.
//!
//! Byte-exact with NeMo's `Normalizer(lang="it", deterministic=True)` on
//! NeMo's own deterministic TN output. Italian is space-delimited, so tokens
//! join with `" "`; it needs no post-processing FST.
//!
//! Grammars in `grammars/it/` are exported from NeMo-text-processing
//! (Apache-2.0, pinned commit `1f1263579fe57ba7ed783cad3dddee710fcc5064`).

use super::{driver, load_gz, AlignedNormalization};
use lazy_static::lazy_static;
use rustfst::prelude::*;

lazy_static! {
    static ref CLASSIFY: VectorFst<TropicalWeight> =
        load_gz(include_bytes!("../../grammars/it/classify.fst.gz"));
    static ref VERBALIZE: VectorFst<TropicalWeight> =
        load_gz(include_bytes!("../../grammars/it/verbalize.fst.gz"));
}

/// Normalize written-form Italian (it) to spoken form.
///
/// ```
/// # #[cfg(feature = "fst-engine")]
/// # {
/// use text_processing_rs::fst::it;
/// assert_eq!(it::normalize("3,23€"), "tre euro ventitre centesimi");
/// assert_eq!(it::normalize("l'evento inizia alle 16:00"), "l'evento inizia alle sedici");
/// # }
/// ```
pub fn normalize(input: &str) -> String {
    driver::normalize(&CLASSIFY, &VERBALIZE, None, input, " ", "it")
}

/// Normalize Italian text and retain the source span for every output token.
pub fn normalize_aligned(input: &str) -> Option<AlignedNormalization> {
    driver::normalize_aligned(&CLASSIFY, &VERBALIZE, None, input, " ", "it")
}
