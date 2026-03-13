//! German inverse text normalization tests.
//!
//! Test cases sourced from NVIDIA NeMo text processing:
//! https://github.com/NVIDIA/NeMo-text-processing

mod common;

use std::path::Path;
use text_processing_rs::normalize_with_lang;

fn normalize_de(input: &str) -> String {
    normalize_with_lang(input, "de")
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
    let results = common::run_test_file(Path::new("tests/data/de/cardinal.txt"), normalize_de);
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
    let results = common::run_test_file(Path::new("tests/data/de/ordinal.txt"), normalize_de);
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
    let results = common::run_test_file(Path::new("tests/data/de/decimal.txt"), normalize_de);
    println!(
        "decimal: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_money() {
    let results = common::run_test_file(Path::new("tests/data/de/money.txt"), normalize_de);
    println!(
        "money: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_date() {
    let results = common::run_test_file(Path::new("tests/data/de/date.txt"), normalize_de);
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
    let results = common::run_test_file(Path::new("tests/data/de/time.txt"), normalize_de);
    println!(
        "time: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_measure() {
    let results = common::run_test_file(Path::new("tests/data/de/measure.txt"), normalize_de);
    println!(
        "measure: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_electronic() {
    let results = common::run_test_file(Path::new("tests/data/de/electronic.txt"), normalize_de);
    println!(
        "electronic: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_telephone() {
    let results = common::run_test_file(Path::new("tests/data/de/telephone.txt"), normalize_de);
    println!(
        "telephone: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_whitelist() {
    let results = common::run_test_file(Path::new("tests/data/de/whitelist.txt"), normalize_de);
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
    let results = common::run_test_file(Path::new("tests/data/de/word.txt"), normalize_de);
    println!(
        "word: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_fraction() {
    let results = common::run_test_file(Path::new("tests/data/de/fraction.txt"), normalize_de);
    println!(
        "fraction: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
}
