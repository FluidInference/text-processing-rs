//! French inverse text normalization tests.
//!
//! Test cases sourced from NVIDIA NeMo text processing:
//! https://github.com/NVIDIA/NeMo-text-processing

mod common;

use std::path::Path;
use text_processing_rs::normalize_with_lang;

fn normalize_fr(input: &str) -> String {
    normalize_with_lang(input, "fr")
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
    let results = common::run_test_file(Path::new("tests/data/fr/cardinal.txt"), normalize_fr);
    println!(
        "cardinal: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_money() {
    let results = common::run_test_file(Path::new("tests/data/fr/money.txt"), normalize_fr);
    println!(
        "money: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_ordinal() {
    let results = common::run_test_file(Path::new("tests/data/fr/ordinal.txt"), normalize_fr);
    println!(
        "ordinal: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_time() {
    let results = common::run_test_file(Path::new("tests/data/fr/time.txt"), normalize_fr);
    println!(
        "time: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_date() {
    let results = common::run_test_file(Path::new("tests/data/fr/date.txt"), normalize_fr);
    println!(
        "date: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_decimal() {
    let results = common::run_test_file(Path::new("tests/data/fr/decimal.txt"), normalize_fr);
    println!(
        "decimal: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_measure() {
    let results = common::run_test_file(Path::new("tests/data/fr/measure.txt"), normalize_fr);
    println!(
        "measure: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_telephone() {
    let results = common::run_test_file(Path::new("tests/data/fr/telephone.txt"), normalize_fr);
    println!(
        "telephone: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_electronic() {
    let results = common::run_test_file(Path::new("tests/data/fr/electronic.txt"), normalize_fr);
    println!(
        "electronic: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_whitelist() {
    let results = common::run_test_file(Path::new("tests/data/fr/whitelist.txt"), normalize_fr);
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
    let results = common::run_test_file(Path::new("tests/data/fr/word.txt"), normalize_fr);
    println!(
        "word: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
}
