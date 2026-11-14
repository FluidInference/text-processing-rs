//! English (en) text normalization via NeMo's compiled grammars.
//!
//! Byte-exact with NeMo's `Normalizer(lang="en", deterministic=True)`:
//! 506/506 on NeMo's own deterministic TN output. English is space-delimited (join `" "`) and applies a post-processing FST.
//!
//! Grammars in `grammars/en/` are exported from NeMo-text-processing
//! (Apache-2.0, pinned commit `1f1263579fe57ba7ed783cad3dddee710fcc5064`).

use super::{driver, load_gz, AlignedNormalization};
use lazy_static::lazy_static;
use rustfst::prelude::*;

lazy_static! {
    static ref CLASSIFY: VectorFst<TropicalWeight> =
        load_gz(include_bytes!("../../grammars/en/classify.fst.gz"));
    static ref VERBALIZE: VectorFst<TropicalWeight> =
        load_gz(include_bytes!("../../grammars/en/verbalize.fst.gz"));
    static ref POST: VectorFst<TropicalWeight> =
        load_gz(include_bytes!("../../grammars/en/postprocess.fst.gz"));
    static ref ITN_CLASSIFY: VectorFst<TropicalWeight> =
        load_gz(include_bytes!("../../grammars/en/itn_classify.fst.gz"));
    static ref ITN_VERBALIZE: VectorFst<TropicalWeight> =
        load_gz(include_bytes!("../../grammars/en/itn_verbalize.fst.gz"));
}

/// Normalize written-form English (en) to spoken form.
///
/// ```
/// # #[cfg(feature = "fst-engine")]
/// # {
/// use text_processing_rs::fst::en;
/// assert_eq!(en::normalize("1"), "one");
/// assert_eq!(en::normalize("$2"), "two dollars");
/// # }
/// ```
pub fn normalize(input: &str) -> String {
    driver::normalize(&CLASSIFY, &VERBALIZE, Some(&POST), input, " ", "en")
}

/// Normalize English text and retain the source span for every output token.
pub fn normalize_aligned(input: &str) -> Option<AlignedNormalization> {
    driver::normalize_aligned(&CLASSIFY, &VERBALIZE, Some(&POST), input, " ", "en")
}

/// Inverse-normalize spoken English (en) to written form, byte-exact with NeMo's
/// `InverseNormalizer(lang="en", input_case="cased")`.
///
/// ```
/// # #[cfg(feature = "fst-engine")]
/// # {
/// use text_processing_rs::fst::en;
/// assert_eq!(en::inverse_normalize("two dollars"), "$2");
/// # }
/// ```
pub fn inverse_normalize(input: &str) -> String {
    driver::inverse_normalize(&ITN_CLASSIFY, &ITN_VERBALIZE, input, " ", "en")
}
