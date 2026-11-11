//! English text normalization tests.
//!
//! Tests for written → spoken conversion (the reverse of ITN).

mod common;

use std::path::Path;
use text_processing_rs::{tn_normalize, tn_normalize_sentence};

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
    let results = common::run_test_file(Path::new("tests/data/en/tn_cardinal.txt"), tn_normalize);
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
    let results = common::run_test_file(Path::new("tests/data/en/tn_ordinal.txt"), tn_normalize);
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
    let results = common::run_test_file(Path::new("tests/data/en/tn_decimal.txt"), tn_normalize);
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
    let results = common::run_test_file(Path::new("tests/data/en/tn_money.txt"), tn_normalize);
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
    let results = common::run_test_file(Path::new("tests/data/en/tn_measure.txt"), tn_normalize);
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
    let results = common::run_test_file(Path::new("tests/data/en/tn_electronic.txt"), tn_normalize);
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
    let results = common::run_test_file(Path::new("tests/data/en/tn_telephone.txt"), tn_normalize);
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
    let results = common::run_test_file(Path::new("tests/data/en/tn_whitelist.txt"), tn_normalize);
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
    use text_processing_rs::normalize;

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

/// Release-sanity check for issue #16: the public surface that hongbo-miao
/// reported missing in crates.io 0.1.0 must compile and behave correctly
/// against the current version. If this test stops compiling, the API
/// surface regressed and consumers will hit `unresolved import`.
#[test]
fn test_issue_16_tn_normalize_sentence_public_api() {
    // Reporter's exact snippet (https://github.com/FluidInference/text-processing-rs/issues/16)
    let normalized_text = tn_normalize_sentence("I have twenty one apples.");
    // The function must exist, be callable, and return a non-empty result.
    // Behavioural assertion: the cardinal "21" is the only normalizable
    // span here, and TN should leave the rest of the sentence intact.
    assert!(
        normalized_text.contains("twenty one") || normalized_text.contains("21"),
        "tn_normalize_sentence dropped or mangled the input: {}",
        normalized_text
    );
}

/// Regression: PR #25 introduced a pretokenizer that splits trailing
/// punctuation off of words so ITN can match `"twenty one,"` as
/// `"twenty one"`. The shared sentence loop must rejoin pretokens using
/// their original separator (no inserted whitespace), otherwise the TN
/// whitelist sees `"Dr ."` instead of `"Dr."` and abbreviation entries
/// like `"e.g."` / `"Prof."` / `"Inc."` stop matching. Reported by Devin
/// AI on PR #25:
/// https://github.com/FluidInference/text-processing-rs/pull/25#pullrequestreview-4178192149
#[test]
fn test_pr25_tn_abbreviation_regression() {
    // Both-form whitelist entries (`Dr` / `Dr.`, `vs` / `vs.`): the period
    // must be consumed by the abbreviation, not left orphaned.
    assert_eq!(
        tn_normalize_sentence("I see Dr. Smith today."),
        "I see doctor Smith today."
    );
    assert_eq!(tn_normalize_sentence("vs. them"), "versus them");

    // Period-only whitelist entries: only match when the trailing period
    // is preserved across the pretokenizer split.
    assert_eq!(
        tn_normalize_sentence("e.g. she is here"),
        "for example she is here"
    );
    assert_eq!(
        tn_normalize_sentence("Inc. and Co."),
        "incorporated and company"
    );
    assert_eq!(tn_normalize_sentence("Prof. Jones"), "professor Jones");
}

#[test]
fn test_saint_only_before_a_name() {
    // "St." before a name is "saint"; after a street name it is the street
    // abbreviation, which NeMo leaves as written.
    let cases = [
        ("St. Louis is far.", "saint Louis is far."),
        (
            "We visited St. Paul's Cathedral.",
            "We visited saint Paul's Cathedral.",
        ),
        ("Main St.", "Main St."),
        ("I live on Main St.", "I live on Main St."),
        ("Meet me at Elm St at noon.", "Meet me at Elm St at noon."),
        (
            "He lives on Baker St in London.",
            "He lives on Baker St in London.",
        ),
    ];
    for (input, expected) in cases {
        assert_eq!(tn_normalize_sentence(input), expected, "input: {input}");
    }
}

#[test]
fn test_tn_fraction() {
    let results = common::run_test_file(Path::new("tests/data/en/tn_fraction.txt"), tn_normalize);
    println!(
        "tn_fraction: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
    assert!(
        results.failures.is_empty(),
        "{} fraction TN tests failed",
        results.failures.len()
    );
}

#[test]
fn test_tn_math() {
    let results = common::run_test_file(Path::new("tests/data/en/tn_math.txt"), tn_normalize);
    println!(
        "tn_math: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
    assert!(
        results.failures.is_empty(),
        "{} math TN tests failed",
        results.failures.len()
    );
}

#[test]
fn test_tn_roman() {
    let results = common::run_test_file(Path::new("tests/data/en/tn_roman.txt"), tn_normalize);
    println!(
        "tn_roman: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
    assert!(
        results.failures.is_empty(),
        "{} roman TN tests failed",
        results.failures.len()
    );
}

#[test]
fn test_tn_range() {
    let results = common::run_test_file(Path::new("tests/data/en/tn_range.txt"), tn_normalize);
    println!(
        "tn_range: {}/{} passed ({} failures)",
        results.passed,
        results.total,
        results.failures.len()
    );
    print_failures(&results);
    assert!(
        results.failures.is_empty(),
        "{} range TN tests failed",
        results.failures.len()
    );
}

// FluidAudio #972: roman-numeral list markers are an opt-in extension beyond
// NeMo (`roman_enumerators`); the default path must keep passing them through.

#[test]
fn test_roman_enumerators_default_unchanged() {
    use text_processing_rs::tn_normalize_sentence_lang;
    assert_eq!(
        tn_normalize_sentence_lang("(i) pay rent; (ii) leave", "en"),
        "(i) pay rent; (ii) leave"
    );
}

#[test]
fn test_roman_enumerators_opt_in() {
    use text_processing_rs::{tn_normalize_sentence_lang_with_options, NormalizeOptions};
    let opts = NormalizeOptions::new().with_roman_enumerators(true);
    assert_eq!(
        tn_normalize_sentence_lang_with_options(
            "The tenant shall: (i) pay $5; (ii) keep the peace; (iv) vacate on notice.",
            "en",
            opts
        ),
        "The tenant shall: (one) pay five dollars; (two) keep the peace; (four) vacate on notice."
    );
    // Stand-alone ambiguous markers and prose are untouched even when opted in.
    assert_eq!(
        tn_normalize_sentence_lang_with_options("morphine (IV) fluids", "en", opts),
        "morphine (IV) fluids"
    );
    assert_eq!(
        tn_normalize_sentence_lang_with_options("mix it, did I?", "en", opts),
        "mix it, did I?"
    );
    // English-only: other languages ignore the flag.
    assert_eq!(
        tn_normalize_sentence_lang_with_options("(ii) chats", "fr", opts),
        "(ii) chats"
    );
}

#[test]
fn test_sentence_final_period_after_money() {
    // "$12.50." used to read as a decimal ("twelve point five zero dollars")
    // and lose the sentence's period (#96).
    assert_eq!(
        tn_normalize_sentence("It costs $12.50."),
        "It costs twelve dollars fifty cents."
    );
    assert_eq!(tn_normalize_sentence("I paid $5."), "I paid five dollars.");
    // An abbreviation keeps its own period.
    assert_eq!(
        tn_normalize_sentence("Meet at 5:30 p.m. today."),
        "Meet at five thirty PM today."
    );
    assert_eq!(
        tn_normalize_sentence("Dr. Smith paid $5."),
        "doctor Smith paid five dollars."
    );
}

// #101: sentence mode read ordinary words as numbers or units (Gutenberg prose).
#[test]
fn test_sentence_prose_false_positives() {
    use text_processing_rs::tn_normalize_sentence_lang;
    let cases = [
        // "did" is not the numeral DID, after a name or otherwise
        ("The Mouse did not answer.", "The Mouse did not answer."),
        ("It did so indeed.", "It did so indeed."),
        ("Louis XVI was king.", "Louis sixteenth was king."),
        // bare "in." and "No." are words; "No." before a number still reads "number"
        (
            "And she opened the door and went in.",
            "And she opened the door and went in.",
        ),
        ("\"No.\"", "\"No.\""),
        (
            "He lives at No. 10 Downing Street.",
            "He lives at number ten Downing Street.",
        ),
        // a year keeps the comma after it and reads as a year
        (
            "On the 20th of July, 1866, the steamer",
            "On the twentieth of July, eighteen sixty six, the steamer",
        ),
        ("He paid 1866, then", "He paid eighteen sixty six, then"),
        (
            "a pressure of 97,500 lbs",
            "a pressure of ninety seven thousand five hundred pounds",
        ),
        // degrees and primes, as NeMo reads them
        ("in 42° 15′ N.", "in forty two degrees fifteen′ N."),
        ("it was 42° today", "it was forty two degrees today"),
        // prose ending in a street-suffix word is not an address
        (
            "rose to the height of 120 feet; thus I ascertained its way",
            "rose to the height of one hundred and twenty feet; thus I ascertained its way",
        ),
        ("123 Main St", "one twenty three Main Street"),
    ];
    for (input, expected) in cases {
        assert_eq!(
            tn_normalize_sentence_lang(input, "en"),
            expected,
            "input: {input}"
        );
    }
}
