//! French (fr) text normalization via NeMo's compiled grammars.
//!
//! Byte-exact with NeMo's `Normalizer(lang="fr", deterministic=True)`: on
//! NeMo's own fr TN test fixtures this scores 117/117. French is
//! space-delimited, so verbalized tokens are joined with `" "`; fr needs no
//! post-processing FST.
//!
//! Grammars in `grammars/fr/` are exported from NeMo-text-processing
//! (Apache-2.0, pinned commit `1f1263579fe57ba7ed783cad3dddee710fcc5064`).

use super::driver;
use lazy_static::lazy_static;
use rustfst::prelude::*;

lazy_static! {
    static ref CLASSIFY: VectorFst<TropicalWeight> =
        VectorFst::load(include_bytes!("../../grammars/fr/classify.fst"))
            .expect("bundled fr classify.fst is valid OpenFST binary");
    static ref VERBALIZE: VectorFst<TropicalWeight> =
        VectorFst::load(include_bytes!("../../grammars/fr/verbalize.fst"))
            .expect("bundled fr verbalize.fst is valid OpenFST binary");
}

/// Normalize written-form French to spoken form.
///
/// ```
/// # #[cfg(feature = "fst-engine")]
/// # {
/// use text_processing_rs::fst::fr;
/// assert_eq!(fr::normalize("83"), "quatre-vingt-trois");
/// assert_eq!(fr::normalize("02/03/2003"), "deux mars deux mille trois");
/// # }
/// ```
pub fn normalize(input: &str) -> String {
    driver::normalize(&CLASSIFY, &VERBALIZE, input, " ")
}
