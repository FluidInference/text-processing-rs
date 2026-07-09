//! Japanese (ja) text normalization via NeMo's compiled grammars.
//!
//! Byte-exact with NeMo's `Normalizer(lang="ja", deterministic=True)`:
//! 542/542 on NeMo's own deterministic TN output. Japanese has no word spaces, so tokens join with `""`; ja needs no post-processing FST.
//!
//! Grammars in `grammars/ja/` are exported from NeMo-text-processing
//! (Apache-2.0, pinned commit `1f1263579fe57ba7ed783cad3dddee710fcc5064`).

use super::{driver, load_gz};
use lazy_static::lazy_static;
use rustfst::prelude::*;

lazy_static! {
    static ref CLASSIFY: VectorFst<TropicalWeight> =
        load_gz(include_bytes!("../../grammars/ja/classify.fst.gz"));
    static ref VERBALIZE: VectorFst<TropicalWeight> =
        load_gz(include_bytes!("../../grammars/ja/verbalize.fst.gz"));
}

/// Normalize written-form Japanese (ja) to spoken form.
///
/// ```
/// # #[cfg(feature = "fst-engine")]
/// # {
/// use text_processing_rs::fst::ja;
/// assert_eq!(ja::normalize("1"), "一");
/// assert_eq!(ja::normalize("2024/01/30"), "二千二十四年一月三十日");
/// # }
/// ```
pub fn normalize(input: &str) -> String {
    driver::normalize(&CLASSIFY, &VERBALIZE, None, input, "")
}
