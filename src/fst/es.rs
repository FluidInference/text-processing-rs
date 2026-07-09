//! Spanish (es) text normalization via NeMo's compiled grammars.
//!
//! Byte-exact with NeMo's `Normalizer(lang="es", deterministic=True)`:
//! 536/536 on NeMo's own deterministic TN output. Spanish is space-delimited, so tokens join with `" "`; es needs no post-processing FST.
//!
//! Grammars in `grammars/es/` are exported from NeMo-text-processing
//! (Apache-2.0, pinned commit `1f1263579fe57ba7ed783cad3dddee710fcc5064`).

use super::{driver, load_gz};
use lazy_static::lazy_static;
use rustfst::prelude::*;

lazy_static! {
    static ref CLASSIFY: VectorFst<TropicalWeight> =
        load_gz(include_bytes!("../../grammars/es/classify.fst.gz"));
    static ref VERBALIZE: VectorFst<TropicalWeight> =
        load_gz(include_bytes!("../../grammars/es/verbalize.fst.gz"));
}

/// Normalize written-form Spanish (es) to spoken form.
///
/// ```
/// # #[cfg(feature = "fst-engine")]
/// # {
/// use text_processing_rs::fst::es;
/// assert_eq!(es::normalize("2"), "dos");
/// assert_eq!(es::normalize("3"), "tres");
/// # }
/// ```
pub fn normalize(input: &str) -> String {
    driver::normalize(&CLASSIFY, &VERBALIZE, None, input, " ")
}
