//! Hindi (hi) text normalization via NeMo's compiled grammars.
//!
//! Byte-exact with NeMo's `Normalizer(lang="hi", deterministic=True)`:
//! 677/677 on NeMo's own deterministic TN output. Hindi is space-delimited (join `" "`) and applies a post-processing FST.
//!
//! Grammars in `grammars/hi/` are exported from NeMo-text-processing
//! (Apache-2.0, pinned commit `1f1263579fe57ba7ed783cad3dddee710fcc5064`).

use super::{driver, load_gz};
use lazy_static::lazy_static;
use rustfst::prelude::*;

lazy_static! {
    static ref CLASSIFY: VectorFst<TropicalWeight> =
        load_gz(include_bytes!("../../grammars/hi/classify.fst.gz"));
    static ref VERBALIZE: VectorFst<TropicalWeight> =
        load_gz(include_bytes!("../../grammars/hi/verbalize.fst.gz"));
    static ref POST: VectorFst<TropicalWeight> =
        load_gz(include_bytes!("../../grammars/hi/postprocess.fst.gz"));
}

/// Normalize written-form Hindi (hi) to spoken form.
///
/// ```
/// # #[cfg(feature = "fst-engine")]
/// # {
/// use text_processing_rs::fst::hi;
/// assert_eq!(hi::normalize("4 चौके"), "चार चौके");
/// # }
/// ```
pub fn normalize(input: &str) -> String {
    driver::normalize(&CLASSIFY, &VERBALIZE, Some(&POST), input, " ")
}
