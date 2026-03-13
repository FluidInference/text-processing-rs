//! Japanese inverse text normalization tests.
//!
//! Test cases sourced from NVIDIA NeMo text processing:
//! https://github.com/NVIDIA/NeMo-text-processing

mod common;

use std::path::Path;
use text_processing_rs::normalize_with_lang;

fn normalize_ja(input: &str) -> String {
    normalize_with_lang(input, "ja")
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
    let results = common::run_test_file(Path::new("tests/data/ja/cardinal.txt"), normalize_ja);
    println!(
        "cardinal: {}/{} passed ({} failures)",
        results.passed, results.total, results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_ordinal() {
    let results = common::run_test_file(Path::new("tests/data/ja/ordinal.txt"), normalize_ja);
    println!(
        "ordinal: {}/{} passed ({} failures)",
        results.passed, results.total, results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_decimal() {
    let results = common::run_test_file(Path::new("tests/data/ja/decimal.txt"), normalize_ja);
    println!(
        "decimal: {}/{} passed ({} failures)",
        results.passed, results.total, results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_date() {
    let results = common::run_test_file(Path::new("tests/data/ja/date.txt"), normalize_ja);
    println!(
        "date: {}/{} passed ({} failures)",
        results.passed, results.total, results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_time() {
    let results = common::run_test_file(Path::new("tests/data/ja/time.txt"), normalize_ja);
    println!(
        "time: {}/{} passed ({} failures)",
        results.passed, results.total, results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_fraction() {
    let results = common::run_test_file(Path::new("tests/data/ja/fraction.txt"), normalize_ja);
    println!(
        "fraction: {}/{} passed ({} failures)",
        results.passed, results.total, results.failures.len()
    );
    print_failures(&results);
}
