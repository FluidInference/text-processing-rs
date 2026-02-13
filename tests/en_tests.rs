//! English inverse text normalization tests.
//!
//! Test cases sourced from NeMo text processing:
//! https://github.com/NVIDIA/NeMo-text-processing

mod common;

use nemo_text_processing::normalize;
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
