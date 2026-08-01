//! Korean inverse text normalization tests.
//!
//! Sino-Korean numeral handling is ported from the euhadra speech
//! framework's evaluation metrics; date / time / decimal / fraction /
//! ordinal taggers follow the Japanese module layout.

mod common;

use std::path::Path;
use text_processing_rs::normalize_with_lang;

fn normalize_ko(input: &str) -> String {
    normalize_with_lang(input, "ko")
}

fn print_failures(results: &common::TestResults) {
    for f in &results.failures {
        println!(
            "  FAIL: '{}' => '{}' (expected '{}')",
            f.input, f.got, f.expected
        );
    }
}

#[test]
fn test_cardinal() {
    let results = common::run_test_file(Path::new("tests/data/ko/cardinal.txt"), normalize_ko);
    println!(
        "cardinal: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_ordinal() {
    let results = common::run_test_file(Path::new("tests/data/ko/ordinal.txt"), normalize_ko);
    println!(
        "ordinal: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_decimal() {
    let results = common::run_test_file(Path::new("tests/data/ko/decimal.txt"), normalize_ko);
    println!(
        "decimal: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_date() {
    let results = common::run_test_file(Path::new("tests/data/ko/date.txt"), normalize_ko);
    println!(
        "date: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_time() {
    let results = common::run_test_file(Path::new("tests/data/ko/time.txt"), normalize_ko);
    println!(
        "time: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_fraction() {
    let results = common::run_test_file(Path::new("tests/data/ko/fraction.txt"), normalize_ko);
    println!(
        "fraction: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
}
