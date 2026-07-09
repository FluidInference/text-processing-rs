//! Byte-exact parity of the FST engine against NeMo's own TN fixtures.
//!
//! Fixtures in `tests/fixtures/<lang>/` are copied verbatim from
//! NeMo-text-processing (Apache-2.0, pinned commit
//! `1f1263579fe57ba7ed783cad3dddee710fcc5064`), so this runs in CI without a
//! NeMo checkout. Each language reaches 100% because NeMo's own normalizer
//! reproduces these fixtures exactly (single deterministic mode).
//!
//! Requires the `fst-engine` feature: `cargo test --features fst-engine`.
#![cfg(feature = "fst-engine")]

mod common;

use std::fs;
use std::path::Path;
use text_processing_rs::fst;

/// Assert every `test_cases_*.txt` under `tests/fixtures/<lang>/` normalizes
/// byte-for-byte, and that a meaningful number of cases actually ran.
fn assert_lang_parity(lang: &str, min_cases: usize, normalize: fn(&str) -> String) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(lang);
    let mut total = 0;
    for entry in fs::read_dir(&dir).expect("fixtures dir") {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if !name.starts_with("test_cases_") {
            continue;
        }
        total += common::parse_test_file(&path).len();
        common::assert_test_file(&path, normalize);
    }
    assert!(
        total >= min_cases,
        "{lang}: expected >= {min_cases} cases, saw only {total}"
    );
}

#[test]
fn zh_tn_matches_nemo() {
    assert_lang_parity("zh", 300, fst::zh::normalize);
}

#[test]
fn fr_tn_matches_nemo() {
    assert_lang_parity("fr", 100, fst::fr::normalize);
}
