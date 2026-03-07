//! English text normalization tests.
//!
//! Tests for written → spoken conversion (the reverse of ITN).

mod common;

use nemo_text_processing::{tn_normalize, tn_normalize_sentence};
use std::path::Path;

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
    let results =
        common::run_test_file(Path::new("tests/data/en/tn_cardinal.txt"), tn_normalize);
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
    let results =
        common::run_test_file(Path::new("tests/data/en/tn_ordinal.txt"), tn_normalize);
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
    let results =
        common::run_test_file(Path::new("tests/data/en/tn_decimal.txt"), tn_normalize);
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
    let results =
        common::run_test_file(Path::new("tests/data/en/tn_money.txt"), tn_normalize);
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
    let results =
        common::run_test_file(Path::new("tests/data/en/tn_measure.txt"), tn_normalize);
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
    let results =
        common::run_test_file(Path::new("tests/data/en/tn_electronic.txt"), tn_normalize);
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
    let results =
        common::run_test_file(Path::new("tests/data/en/tn_telephone.txt"), tn_normalize);
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
    let results =
        common::run_test_file(Path::new("tests/data/en/tn_whitelist.txt"), tn_normalize);
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
    use nemo_text_processing::normalize;

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
