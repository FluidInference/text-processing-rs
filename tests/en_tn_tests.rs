//! English text normalization tests.
//!
//! Tests for written → spoken conversion (the reverse of ITN).

mod common;

use std::path::Path;
use text_processing_rs::{tn_normalize, tn_normalize_sentence};

fn print_failures(results: &common::TestResults) {
    for f in &results.failures {
        println!(
            "  FAIL: '{}' => '{}' (expected '{}')",
            f.input, f.got, f.expected
        );
    }
}

#[test]
fn test_tn_cardinal() {
    let results = common::run_test_file(Path::new("tests/data/en/tn_cardinal.txt"), tn_normalize);
    println!(
        "tn_cardinal: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
    assert!(
        results.failures.is_empty(),
        "{} cardinal TN tests failed",
        results.failures.len()
    );
}

#[test]
fn test_tn_ordinal() {
    let results = common::run_test_file(Path::new("tests/data/en/tn_ordinal.txt"), tn_normalize);
    println!(
        "tn_ordinal: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
    assert!(
        results.failures.is_empty(),
        "{} ordinal TN tests failed",
        results.failures.len()
    );
}

#[test]
fn test_tn_decimal() {
    let results = common::run_test_file(Path::new("tests/data/en/tn_decimal.txt"), tn_normalize);
    println!(
        "tn_decimal: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
    assert!(
        results.failures.is_empty(),
        "{} decimal TN tests failed",
        results.failures.len()
    );
}

#[test]
fn test_tn_money() {
    let results = common::run_test_file(Path::new("tests/data/en/tn_money.txt"), tn_normalize);
    println!(
        "tn_money: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
    assert!(
        results.failures.is_empty(),
        "{} money TN tests failed",
        results.failures.len()
    );
}

#[test]
fn test_tn_time() {
    let results = common::run_test_file(Path::new("tests/data/en/tn_time.txt"), tn_normalize);
    println!(
        "tn_time: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
    assert!(
        results.failures.is_empty(),
        "{} time TN tests failed",
        results.failures.len()
    );
}

#[test]
fn test_tn_date() {
    let results = common::run_test_file(Path::new("tests/data/en/tn_date.txt"), tn_normalize);
    println!(
        "tn_date: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
    assert!(
        results.failures.is_empty(),
        "{} date TN tests failed",
        results.failures.len()
    );
}

#[test]
fn test_tn_measure() {
    let results = common::run_test_file(Path::new("tests/data/en/tn_measure.txt"), tn_normalize);
    println!(
        "tn_measure: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
    assert!(
        results.failures.is_empty(),
        "{} measure TN tests failed",
        results.failures.len()
    );
}

#[test]
fn test_tn_electronic() {
    let results = common::run_test_file(Path::new("tests/data/en/tn_electronic.txt"), tn_normalize);
    println!(
        "tn_electronic: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
    assert!(
        results.failures.is_empty(),
        "{} electronic TN tests failed",
        results.failures.len()
    );
}

#[test]
fn test_tn_telephone() {
    let results = common::run_test_file(Path::new("tests/data/en/tn_telephone.txt"), tn_normalize);
    println!(
        "tn_telephone: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
    assert!(
        results.failures.is_empty(),
        "{} telephone TN tests failed",
        results.failures.len()
    );
}

#[test]
fn test_tn_whitelist() {
    let results = common::run_test_file(Path::new("tests/data/en/tn_whitelist.txt"), tn_normalize);
    println!(
        "tn_whitelist: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
    assert!(
        results.failures.is_empty(),
        "{} whitelist TN tests failed",
        results.failures.len()
    );
}

#[test]
fn test_tn_sentence_mixed() {
    assert_eq!(
        tn_normalize_sentence("I paid $5 for 23 items"),
        "I paid five dollars for twenty three items"
    );
    assert_eq!(tn_normalize_sentence("hello world"), "hello world");
    assert_eq!(tn_normalize_sentence(""), "");
}

#[test]
fn test_tn_roundtrip_cardinal() {
    // Verify that ITN(TN(written)) ≈ written for simple cardinals
    use text_processing_rs::normalize;

    let cases = &["123", "42", "1000", "21"];
    for &written in cases {
        let spoken = tn_normalize(written);
        let back = normalize(&spoken);
        assert_eq!(
            back, written,
            "Roundtrip failed: {} → {} → {}",
            written, spoken, back
        );
    }
}

/// Release-sanity check for issue #16: the public surface that hongbo-miao
/// reported missing in crates.io 0.1.0 must compile and behave correctly
/// against the current version. If this test stops compiling, the API
/// surface regressed and consumers will hit `unresolved import`.
#[test]
fn test_issue_16_tn_normalize_sentence_public_api() {
    // Reporter's exact snippet (https://github.com/FluidInference/text-processing-rs/issues/16)
    let normalized_text = tn_normalize_sentence("I have twenty one apples.");
    // The function must exist, be callable, and return a non-empty result.
    // Behavioural assertion: the cardinal "21" is the only normalizable
    // span here, and TN should leave the rest of the sentence intact.
    assert!(
        normalized_text.contains("twenty one") || normalized_text.contains("21"),
        "tn_normalize_sentence dropped or mangled the input: {}",
        normalized_text
    );
}

/// Regression: PR #25 introduced a pretokenizer that splits trailing
/// punctuation off of words so ITN can match `"twenty one,"` as
/// `"twenty one"`. The shared sentence loop must rejoin pretokens using
/// their original separator (no inserted whitespace), otherwise the TN
/// whitelist sees `"Dr ."` instead of `"Dr."` and abbreviation entries
/// like `"e.g."` / `"Prof."` / `"Inc."` stop matching. Reported by Devin
/// AI on PR #25:
/// https://github.com/FluidInference/text-processing-rs/pull/25#pullrequestreview-4178192149
#[test]
fn test_pr25_tn_abbreviation_regression() {
    // Both-form whitelist entries (`Dr` / `Dr.`, `vs` / `vs.`): the period
    // must be consumed by the abbreviation, not left orphaned.
    assert_eq!(
        tn_normalize_sentence("I see Dr. Smith today."),
        "I see doctor Smith today."
    );
    assert_eq!(tn_normalize_sentence("vs. them"), "versus them");

    // Period-only whitelist entries: only match when the trailing period
    // is preserved across the pretokenizer split.
    assert_eq!(
        tn_normalize_sentence("e.g. she is here"),
        "for example she is here"
    );
    assert_eq!(
        tn_normalize_sentence("Inc. and Co."),
        "incorporated and company"
    );
    assert_eq!(tn_normalize_sentence("Prof. Jones"), "professor Jones");
}

#[test]
fn test_tn_fraction() {
    let results = common::run_test_file(Path::new("tests/data/en/tn_fraction.txt"), tn_normalize);
    println!(
        "tn_fraction: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
    assert!(
        results.failures.is_empty(),
        "{} fraction TN tests failed",
        results.failures.len()
    );
}

#[test]
fn test_tn_math() {
    let results = common::run_test_file(Path::new("tests/data/en/tn_math.txt"), tn_normalize);
    println!(
        "tn_math: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
    assert!(
        results.failures.is_empty(),
        "{} math TN tests failed",
        results.failures.len()
    );
}
