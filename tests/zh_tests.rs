//! Chinese inverse text normalization tests.
//!
//! Test cases sourced from NVIDIA NeMo text processing:
//! https://github.com/NVIDIA/NeMo-text-processing

mod common;

use std::path::Path;
use text_processing_rs::normalize_with_lang;

fn normalize_zh(input: &str) -> String {
    normalize_with_lang(input, "zh")
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
    let results = common::run_test_file(Path::new("tests/data/zh/cardinal.txt"), normalize_zh);
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
    let results = common::run_test_file(Path::new("tests/data/zh/ordinal.txt"), normalize_zh);
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
    let results = common::run_test_file(Path::new("tests/data/zh/decimal.txt"), normalize_zh);
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
    let results = common::run_test_file(Path::new("tests/data/zh/date.txt"), normalize_zh);
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
    let results = common::run_test_file(Path::new("tests/data/zh/time.txt"), normalize_zh);
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
    let results = common::run_test_file(Path::new("tests/data/zh/fraction.txt"), normalize_zh);
    println!(
        "fraction: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_money() {
    let results = common::run_test_file(Path::new("tests/data/zh/money.txt"), normalize_zh);
    println!(
        "money: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_whitelist() {
    let results = common::run_test_file(Path::new("tests/data/zh/whitelist.txt"), normalize_zh);
    println!(
        "whitelist: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_word() {
    let results = common::run_test_file(Path::new("tests/data/zh/word.txt"), normalize_zh);
    println!(
        "word: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
}
