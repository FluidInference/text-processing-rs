//! German (de) text normalization via NeMo's compiled grammars.
//!
//! Byte-exact with NeMo's `Normalizer(lang="de", deterministic=True)`:
//! 314/314 on NeMo's own deterministic TN output. German is space-delimited, so tokens join with `" "`; de needs no post-processing FST.
//!
//! Grammars in `grammars/de/` are exported from NeMo-text-processing
//! (Apache-2.0, pinned commit `1f1263579fe57ba7ed783cad3dddee710fcc5064`).

use super::{driver, load_gz};
use lazy_static::lazy_static;
use rustfst::prelude::*;

lazy_static! {
    static ref CLASSIFY: VectorFst<TropicalWeight> =
        load_gz(include_bytes!("../../grammars/de/classify.fst.gz"));
    static ref VERBALIZE: VectorFst<TropicalWeight> =
        load_gz(include_bytes!("../../grammars/de/verbalize.fst.gz"));
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
    driver::normalize(&CLASSIFY, &VERBALIZE, None, input, " ")
}
