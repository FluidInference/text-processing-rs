//! Byte-exact parity of the FST engine against NeMo's own zh TN fixtures.
//!
//! Fixtures in `tests/fixtures/zh/` are copied verbatim from
//! NeMo-text-processing (Apache-2.0, pinned commit
//! `1f1263579fe57ba7ed783cad3dddee710fcc5064`,
//! `tests/nemo_text_processing/zh/data_text_normalization/`), so this runs in
//! CI without a NeMo checkout. Each line is `input~expected`.
//!
//! Requires the `fst-engine` feature: `cargo test --features fst-engine`.
#![cfg(feature = "fst-engine")]

use std::fs;
use std::path::Path;
use text_processing_rs::fst::zh;

#[test]
fn zh_tn_matches_nemo_fixtures() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/zh");
    let mut total = 0;
    let mut passed = 0;
    let mut failures = Vec::new();

    for entry in fs::read_dir(&dir).expect("zh fixtures dir") {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if !name.starts_with("test_cases_") {
            continue;
        }
        for line in fs::read_to_string(&path).unwrap_or_default().lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((input, expected)) = line.split_once('~') else {
                continue;
            };
            total += 1;
            let got = zh::normalize(input);
            if got == expected {
                passed += 1;
            } else if failures.len() < 20 {
                failures.push(format!("[{input}] got [{got}] want [{expected}]"));
            }
        }
    }

    assert!(
        total > 300,
        "expected the full zh suite, saw only {total} cases"
    );
    assert_eq!(
        passed,
        total,
        "zh FST parity {passed}/{total}; first failures:\n  {}",
        failures.join("\n  ")
    );
}
