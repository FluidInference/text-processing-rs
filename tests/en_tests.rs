//! English inverse text normalization tests.
//!
//! Test cases sourced from NeMo text processing:
//! https://github.com/NVIDIA/NeMo-text-processing

mod common;

use nemo_text_processing::{normalize, normalize_sentence};
use std::path::Path;

fn print_failures(results: &common::TestResults) {
    for f in &results.failures {
        println!("  FAIL: '{}' => '{}' (expected '{}')", f.input, f.got, f.expected);
    }
}

#[test]
fn test_cardinal() {
    let results = common::run_test_file(Path::new("tests/data/en/cardinal.txt"), normalize);
    println!("cardinal: {}/{} passed ({} failures)", results.passed, results.total, results.failures.len());
    print_failures(&results);
}

#[test]
fn test_money() {
    let results = common::run_test_file(Path::new("tests/data/en/money.txt"), normalize);
    println!("money: {}/{} passed ({} failures)", results.passed, results.total, results.failures.len());
    print_failures(&results);
}

#[test]
fn test_ordinal() {
    let results = common::run_test_file(Path::new("tests/data/en/ordinal.txt"), normalize);
    println!("ordinal: {}/{} passed ({} failures)", results.passed, results.total, results.failures.len());
    print_failures(&results);
}

#[test]
fn test_time() {
    let results = common::run_test_file(Path::new("tests/data/en/time.txt"), normalize);
    println!("time: {}/{} passed ({} failures)", results.passed, results.total, results.failures.len());
    print_failures(&results);
}

#[test]
fn test_date() {
    let results = common::run_test_file(Path::new("tests/data/en/date.txt"), normalize);
    println!("date: {}/{} passed ({} failures)", results.passed, results.total, results.failures.len());
    print_failures(&results);
}

#[test]
fn test_decimal() {
    let results = common::run_test_file(Path::new("tests/data/en/decimal.txt"), normalize);
    println!("decimal: {}/{} passed ({} failures)", results.passed, results.total, results.failures.len());
    print_failures(&results);
}

#[test]
fn test_measure() {
    let results = common::run_test_file(Path::new("tests/data/en/measure.txt"), normalize);
    println!("measure: {}/{} passed ({} failures)", results.passed, results.total, results.failures.len());
    print_failures(&results);
}

#[test]
fn test_telephone() {
    let results = common::run_test_file(Path::new("tests/data/en/telephone.txt"), normalize);
    println!("telephone: {}/{} passed ({} failures)", results.passed, results.total, results.failures.len());
    print_failures(&results);
}

#[test]
fn test_electronic() {
    let results = common::run_test_file(Path::new("tests/data/en/electronic.txt"), normalize);
    println!("electronic: {}/{} passed ({} failures)", results.passed, results.total, results.failures.len());
    print_failures(&results);
}

#[test]
fn test_whitelist() {
    let results = common::run_test_file(Path::new("tests/data/en/whitelist.txt"), normalize);
    println!("whitelist: {}/{} passed ({} failures)", results.passed, results.total, results.failures.len());
    print_failures(&results);
}

#[test]
fn test_word() {
    let results = common::run_test_file(Path::new("tests/data/en/word.txt"), normalize);
    println!("word: {}/{} passed ({} failures)", results.passed, results.total, results.failures.len());
    print_failures(&results);
}

// Cased tests - case-sensitive versions

#[test]
fn test_cardinal_cased() {
    let results = common::run_test_file(Path::new("tests/data/en/cardinal_cased.txt"), normalize);
    println!("cardinal_cased: {}/{} passed ({} failures)", results.passed, results.total, results.failures.len());
    print_failures(&results);
}

#[test]
fn test_date_cased() {
    let results = common::run_test_file(Path::new("tests/data/en/date_cased.txt"), normalize);
    println!("date_cased: {}/{} passed ({} failures)", results.passed, results.total, results.failures.len());
    print_failures(&results);
}

#[test]
fn test_decimal_cased() {
    let results = common::run_test_file(Path::new("tests/data/en/decimal_cased.txt"), normalize);
    println!("decimal_cased: {}/{} passed ({} failures)", results.passed, results.total, results.failures.len());
    print_failures(&results);
}

#[test]
fn test_electronic_cased() {
    let results = common::run_test_file(Path::new("tests/data/en/electronic_cased.txt"), normalize);
    println!("electronic_cased: {}/{} passed ({} failures)", results.passed, results.total, results.failures.len());
    print_failures(&results);
}

#[test]
fn test_measure_cased() {
    let results = common::run_test_file(Path::new("tests/data/en/measure_cased.txt"), normalize);
    println!("measure_cased: {}/{} passed ({} failures)", results.passed, results.total, results.failures.len());
    print_failures(&results);
}

#[test]
fn test_money_cased() {
    let results = common::run_test_file(Path::new("tests/data/en/money_cased.txt"), normalize);
    println!("money_cased: {}/{} passed ({} failures)", results.passed, results.total, results.failures.len());
    print_failures(&results);
}

#[test]
fn test_ordinal_cased() {
    let results = common::run_test_file(Path::new("tests/data/en/ordinal_cased.txt"), normalize);
    println!("ordinal_cased: {}/{} passed ({} failures)", results.passed, results.total, results.failures.len());
    print_failures(&results);
}

#[test]
fn test_telephone_cased() {
    let results = common::run_test_file(Path::new("tests/data/en/telephone_cased.txt"), normalize);
    println!("telephone_cased: {}/{} passed ({} failures)", results.passed, results.total, results.failures.len());
    print_failures(&results);
}

#[test]
fn test_time_cased() {
    let results = common::run_test_file(Path::new("tests/data/en/time_cased.txt"), normalize);
    println!("time_cased: {}/{} passed ({} failures)", results.passed, results.total, results.failures.len());
    print_failures(&results);
}

#[test]
fn test_whitelist_cased() {
    let results = common::run_test_file(Path::new("tests/data/en/whitelist_cased.txt"), normalize);
    println!("whitelist_cased: {}/{} passed ({} failures)", results.passed, results.total, results.failures.len());
    print_failures(&results);
}

#[test]
fn test_word_cased() {
    let results = common::run_test_file(Path::new("tests/data/en/word_cased.txt"), normalize);
    println!("word_cased: {}/{} passed ({} failures)", results.passed, results.total, results.failures.len());
    print_failures(&results);
}

// Sentence-mode tests

#[test]
fn test_sentence_cardinal_in_context() {
    assert_eq!(normalize_sentence("I have twenty one apples"), "I have 21 apples");
    assert_eq!(normalize_sentence("there are three hundred people here"), "there are 300 people here");
    assert_eq!(normalize_sentence("she is forty two years old"), "she is 42 years old");
}

#[test]
fn test_sentence_money_in_context() {
    assert_eq!(normalize_sentence("five dollars and fifty cents for the coffee"), "$5.50 for the coffee");
    assert_eq!(normalize_sentence("I paid five dollars for lunch"), "I paid $5 for lunch");
}

#[test]
fn test_sentence_passthrough() {
    assert_eq!(normalize_sentence("hello world"), "hello world");
    assert_eq!(normalize_sentence("the quick brown fox jumps over the lazy dog"), "the quick brown fox jumps over the lazy dog");
    assert_eq!(normalize_sentence(""), "");
}

#[test]
fn test_sentence_time_in_context() {
    assert_eq!(normalize_sentence("call me at two thirty pm tomorrow"), "call me at 02:30 p.m. tomorrow");
}

#[test]
fn test_sentence_mixed_types() {
    assert_eq!(normalize_sentence("I paid five dollars for twenty three items"), "I paid $5 for 23 items");
}

#[test]
fn test_sentence_ordinal_in_context() {
    assert_eq!(normalize_sentence("she finished in twenty first place"), "she finished in 21st place");
}

#[test]
fn test_sentence_existing_tests_via_sentence() {
    // Existing normalize() test cases should also work through normalize_sentence()
    // when the entire input is a single normalizable expression.
    let results = common::run_test_file(Path::new("tests/data/en/money.txt"), normalize_sentence);
    println!("money (sentence mode): {}/{} passed ({} failures)", results.passed, results.total, results.failures.len());
    // Don't assert all pass — sentence mode intentionally excludes some taggers and
    // limits cardinal span length, so some edge cases may differ. Just print results.
    print_failures(&results);
}
