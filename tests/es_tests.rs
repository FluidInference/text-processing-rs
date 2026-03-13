//! Spanish inverse text normalization tests.
//!
//! Test cases sourced from NVIDIA NeMo text processing:
//! https://github.com/NVIDIA/NeMo-text-processing

mod common;

use std::path::Path;
use text_processing_rs::normalize_with_lang;

fn normalize_es(input: &str) -> String {
    normalize_with_lang(input, "es")
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
    let results = common::run_test_file(Path::new("tests/data/es/cardinal.txt"), normalize_es);
    println!(
        "cardinal: {}/{} passed ({} failures)",
        results.passed, results.total, results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_ordinal() {
    let results = common::run_test_file(Path::new("tests/data/es/ordinal.txt"), normalize_es);
    println!(
        "ordinal: {}/{} passed ({} failures)",
        results.passed, results.total, results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_decimal() {
    let results = common::run_test_file(Path::new("tests/data/es/decimal.txt"), normalize_es);
    println!(
        "decimal: {}/{} passed ({} failures)",
        results.passed, results.total, results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_money() {
    let results = common::run_test_file(Path::new("tests/data/es/money.txt"), normalize_es);
    println!(
        "money: {}/{} passed ({} failures)",
        results.passed, results.total, results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_date() {
    let results = common::run_test_file(Path::new("tests/data/es/date.txt"), normalize_es);
    println!(
        "date: {}/{} passed ({} failures)",
        results.passed, results.total, results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_time() {
    let results = common::run_test_file(Path::new("tests/data/es/time.txt"), normalize_es);
    println!(
        "time: {}/{} passed ({} failures)",
        results.passed, results.total, results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_measure() {
    let results = common::run_test_file(Path::new("tests/data/es/measure.txt"), normalize_es);
    println!(
        "measure: {}/{} passed ({} failures)",
        results.passed, results.total, results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_electronic() {
    let results = common::run_test_file(Path::new("tests/data/es/electronic.txt"), normalize_es);
    println!(
        "electronic: {}/{} passed ({} failures)",
        results.passed, results.total, results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_telephone() {
    let results = common::run_test_file(Path::new("tests/data/es/telephone.txt"), normalize_es);
    println!(
        "telephone: {}/{} passed ({} failures)",
        results.passed, results.total, results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_whitelist() {
    let results = common::run_test_file(Path::new("tests/data/es/whitelist.txt"), normalize_es);
    println!(
        "whitelist: {}/{} passed ({} failures)",
        results.passed, results.total, results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_word() {
    let results = common::run_test_file(Path::new("tests/data/es/word.txt"), normalize_es);
    println!(
        "word: {}/{} passed ({} failures)",
        results.passed, results.total, results.failures.len()
    );
    print_failures(&results);
}

#[test]
fn test_fraction() {
    let results = common::run_test_file(Path::new("tests/data/es/fraction.txt"), normalize_es);
    println!(
        "fraction: {}/{} passed ({} failures)",
        results.passed, results.total, results.failures.len()
    );
    print_failures(&results);
}
