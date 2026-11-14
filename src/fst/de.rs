//! German (de) text normalization via NeMo's compiled grammars.
//!
//! Byte-exact with NeMo's `Normalizer(lang="de", deterministic=True)`:
//! 314/314 on NeMo's own deterministic TN output. German is space-delimited, so tokens join with `" "`; de needs no post-processing FST.
//!
//! Grammars in `grammars/de/` are exported from NeMo-text-processing
//! (Apache-2.0, pinned commit `1f1263579fe57ba7ed783cad3dddee710fcc5064`).

use super::{driver, load_gz, AlignedNormalization};
use lazy_static::lazy_static;
use rustfst::prelude::*;

lazy_static! {
    static ref CLASSIFY: VectorFst<TropicalWeight> =
        load_gz(include_bytes!("../../grammars/de/classify.fst.gz"));
    static ref VERBALIZE: VectorFst<TropicalWeight> =
        load_gz(include_bytes!("../../grammars/de/verbalize.fst.gz"));
    static ref ITN_CLASSIFY: VectorFst<TropicalWeight> =
        load_gz(include_bytes!("../../grammars/de/itn_classify.fst.gz"));
    static ref ITN_VERBALIZE: VectorFst<TropicalWeight> =
        load_gz(include_bytes!("../../grammars/de/itn_verbalize.fst.gz"));
}

/// Normalize written-form German (de) to spoken form.
///
/// ```
/// # #[cfg(feature = "fst-engine")]
/// # {
/// use text_processing_rs::fst::de;
/// assert_eq!(de::normalize("0"), "null");
/// assert_eq!(de::normalize("1"), "eins");
/// # }
/// ```
pub fn normalize(input: &str) -> String {
    driver::normalize(&CLASSIFY, &VERBALIZE, None, input, " ", "de")
}

/// Normalize German text and retain the source span for every output token.
pub fn normalize_aligned(input: &str) -> Option<AlignedNormalization> {
    driver::normalize_aligned(&CLASSIFY, &VERBALIZE, None, input, " ", "de")
}

/// Inverse-normalize spoken German (de) to written form, byte-exact with NeMo's
/// `InverseNormalizer(lang="de", input_case="cased")`.
pub fn inverse_normalize(input: &str) -> String {
    driver::inverse_normalize(&ITN_CLASSIFY, &ITN_VERBALIZE, input, " ", "de")
}
