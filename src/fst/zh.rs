//! Mandarin (zh) text normalization via NeMo's compiled grammars.
//!
//! Byte-exact with NeMo's `Normalizer(lang="zh", deterministic=True)`:
//! 367/367 on NeMo's own deterministic TN output. Chinese has no word spaces, so tokens join with `""`; zh needs no post-processing FST.
//!
//! Grammars in `grammars/zh/` are exported from NeMo-text-processing
//! (Apache-2.0, pinned commit `1f1263579fe57ba7ed783cad3dddee710fcc5064`).

use super::{driver, load_gz};
use lazy_static::lazy_static;
use rustfst::prelude::*;

lazy_static! {
    static ref CLASSIFY: VectorFst<TropicalWeight> =
        load_gz(include_bytes!("../../grammars/zh/classify.fst.gz"));
    static ref VERBALIZE: VectorFst<TropicalWeight> =
        load_gz(include_bytes!("../../grammars/zh/verbalize.fst.gz"));
}

/// Normalize written-form Mandarin (zh) to spoken form.
///
/// ```
/// # #[cfg(feature = "fst-engine")]
/// # {
/// use text_processing_rs::fst::zh;
/// assert_eq!(zh::normalize("2024年"), "二零二四年");
/// assert_eq!(zh::normalize("$123"), "一百二十三美元");
/// assert_eq!(zh::normalize("12.5"), "十二点五");
/// # }
/// ```
pub fn normalize(input: &str) -> String {
    driver::normalize(&CLASSIFY, &VERBALIZE, None, input, "")
}
