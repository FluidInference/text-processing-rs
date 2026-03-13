//! Hindi inverse text normalization tests.
//!
//! Test cases sourced from NVIDIA NeMo text processing:
//! https://github.com/NVIDIA/NeMo-text-processing

mod common;

use std::path::Path;
use text_processing_rs::normalize_with_lang;

/// Decompose precomposed Devanagari nukta characters for consistent comparison.
/// Both input normalization (in lib.rs) and expected output may use different
/// Unicode representations of the same character.
fn decompose_nukta(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 16);
    for c in input.chars() {
        match c {
            '\u{0958}' => { out.push('\u{0915}'); out.push('\u{093C}'); }
            '\u{0959}' => { out.push('\u{0916}'); out.push('\u{093C}'); }
            '\u{095A}' => { out.push('\u{0917}'); out.push('\u{093C}'); }
            '\u{095B}' => { out.push('\u{091C}'); out.push('\u{093C}'); }
            '\u{095C}' => { out.push('\u{0921}'); out.push('\u{093C}'); }
            '\u{095D}' => { out.push('\u{0922}'); out.push('\u{093C}'); }
            '\u{095E}' => { out.push('\u{092B}'); out.push('\u{093C}'); }
            '\u{095F}' => { out.push('\u{092F}'); out.push('\u{093C}'); }
            _ => out.push(c),
        }
    }
    out
}

fn normalize_hi(input: &str) -> String {
    normalize_with_lang(input, "hi")
}

/// Compare with nukta normalization on both sides.
fn nukta_eq(got: &str, expected: &str) -> bool {
    decompose_nukta(got) == decompose_nukta(expected)
}

fn run_hi_test(name: &str, file: &str) {
    let results = common::run_test_file_with_compare(
        Path::new(file),
        normalize_hi,
        nukta_eq,
    );
    println!(
        "{}: {}/{} passed ({} failures)",
        name, results.passed, results.total, results.failures.len()
    );
    for f in &results.failures {
        println!(
            "  FAIL: '{}' => '{}' (expected '{}')",
            f.input, f.got, f.expected
        );
    }
}

#[test]
fn test_cardinal() { run_hi_test("cardinal", "tests/data/hi/cardinal.txt"); }

#[test]
fn test_ordinal() { run_hi_test("ordinal", "tests/data/hi/ordinal.txt"); }

#[test]
fn test_decimal() { run_hi_test("decimal", "tests/data/hi/decimal.txt"); }

#[test]
fn test_date() { run_hi_test("date", "tests/data/hi/date.txt"); }

#[test]
fn test_time() { run_hi_test("time", "tests/data/hi/time.txt"); }

#[test]
fn test_fraction() { run_hi_test("fraction", "tests/data/hi/fraction.txt"); }

#[test]
fn test_money() { run_hi_test("money", "tests/data/hi/money.txt"); }

#[test]
fn test_measure() { run_hi_test("measure", "tests/data/hi/measure.txt"); }

#[test]
fn test_whitelist() { run_hi_test("whitelist", "tests/data/hi/whitelist.txt"); }

#[test]
fn test_word() { run_hi_test("word", "tests/data/hi/word.txt"); }

#[test]
fn test_address() { run_hi_test("address", "tests/data/hi/address.txt"); }

#[test]
fn test_telephone() { run_hi_test("telephone", "tests/data/hi/telephone.txt"); }
