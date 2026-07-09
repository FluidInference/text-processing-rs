//! Mandarin (zh) text normalization via NeMo's compiled grammars.
//!
//! Byte-exact with NeMo's `Normalizer(lang="zh", deterministic=True)`: on
//! NeMo's own zh TN test fixtures this scores 353/353. Chinese has no word
//! spaces, so verbalized tokens are joined with `""`; zh needs no
//! post-processing FST.
//!
//! Grammars in `grammars/zh/` are exported from NeMo-text-processing
//! (Apache-2.0, pinned commit `1f1263579fe57ba7ed783cad3dddee710fcc5064`).

use super::driver;
use lazy_static::lazy_static;
use rustfst::prelude::*;

lazy_static! {
    static ref CLASSIFY: VectorFst<TropicalWeight> =
        VectorFst::load(include_bytes!("../../grammars/zh/classify.fst"))
            .expect("bundled zh classify.fst is valid OpenFST binary");
    static ref VERBALIZE: VectorFst<TropicalWeight> =
        VectorFst::load(include_bytes!("../../grammars/zh/verbalize.fst"))
            .expect("bundled zh verbalize.fst is valid OpenFST binary");
}

/// Normalize written-form Mandarin to spoken form.
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
    driver::normalize(&CLASSIFY, &VERBALIZE, input, "")
}
