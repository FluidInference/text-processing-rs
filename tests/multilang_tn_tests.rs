//! Multi-language text normalization integration tests.
//!
//! Tests the language dispatch API: tn_normalize_lang() and tn_normalize_sentence_lang().
//! Ensures all 6 languages are wired up and producing correct output through the public API.

use text_processing_rs::{tn_normalize_lang, tn_normalize_sentence_lang};

// ── French ───────────────────────────────────────────────────────────

#[test]
fn test_fr_cardinal() {
    assert_eq!(tn_normalize_lang("123", "fr"), "cent vingt-trois");
    assert_eq!(tn_normalize_lang("0", "fr"), "zero");
    assert_eq!(tn_normalize_lang("71", "fr"), "soixante et onze");
    assert_eq!(tn_normalize_lang("80", "fr"), "quatre-vingts");
    assert_eq!(tn_normalize_lang("99", "fr"), "quatre-vingt-dix-neuf");
}

#[test]
fn test_fr_money() {
    assert_eq!(
        tn_normalize_lang("5,50 \u{20ac}", "fr"),
        "cinq euros et cinquante centimes"
    );
    assert_eq!(tn_normalize_lang("$1", "fr"), "un dollar");
}

#[test]
fn test_fr_time() {
    assert_eq!(tn_normalize_lang("14:30", "fr"), "quatorze heures trente");
    assert_eq!(tn_normalize_lang("0:00", "fr"), "minuit");
    assert_eq!(tn_normalize_lang("12:00", "fr"), "midi");
}

#[test]
fn test_fr_sentence() {
    assert_eq!(
        tn_normalize_sentence_lang("Il a 123 chats", "fr"),
        "Il a cent vingt-trois chats"
    );
}

// ── Spanish ──────────────────────────────────────────────────────────

#[test]
fn test_es_cardinal() {
    assert_eq!(tn_normalize_lang("123", "es"), "ciento veintitres");
    assert_eq!(tn_normalize_lang("21", "es"), "veintiuno");
    assert_eq!(tn_normalize_lang("100", "es"), "cien");
    assert_eq!(tn_normalize_lang("500", "es"), "quinientos");
}

#[test]
fn test_es_money() {
    assert_eq!(tn_normalize_lang("$5", "es"), "cinco dolares");
    assert_eq!(tn_normalize_lang("$1", "es"), "un dolar");
}

#[test]
fn test_es_time() {
    assert_eq!(tn_normalize_lang("14:30", "es"), "catorce treinta");
    assert_eq!(tn_normalize_lang("0:00", "es"), "medianoche");
    assert_eq!(tn_normalize_lang("12:00", "es"), "mediodia");
}

#[test]
fn test_es_sentence() {
    assert_eq!(
        tn_normalize_sentence_lang("Tengo 21 gatos", "es"),
        "Tengo veintiuno gatos"
    );
}

// ── German ───────────────────────────────────────────────────────────

#[test]
fn test_de_cardinal() {
    assert_eq!(tn_normalize_lang("21", "de"), "einundzwanzig");
    assert_eq!(tn_normalize_lang("123", "de"), "einhundertdreiundzwanzig");
    assert_eq!(
        tn_normalize_lang("2025", "de"),
        "zweitausendfuenfundzwanzig"
    );
}

#[test]
fn test_de_money() {
    assert_eq!(
        tn_normalize_lang("5,50 \u{20ac}", "de"),
        "fuenf euro und fuenfzig cent"
    );
}

#[test]
fn test_de_time() {
    assert_eq!(tn_normalize_lang("14:30", "de"), "vierzehn uhr dreissig");
    assert_eq!(tn_normalize_lang("0:00", "de"), "mitternacht");
    assert_eq!(tn_normalize_lang("12:00", "de"), "mittag");
}

#[test]
fn test_de_sentence() {
    assert_eq!(
        tn_normalize_sentence_lang("Ich habe 123 Katzen", "de"),
        "Ich habe einhundertdreiundzwanzig Katzen"
    );
}

// ── Mandarin Chinese ─────────────────────────────────────────────────

#[test]
fn test_zh_cardinal() {
    assert_eq!(tn_normalize_lang("123", "zh"), "yi bai er shi san");
    assert_eq!(tn_normalize_lang("10000", "zh"), "yi wan");
    assert_eq!(tn_normalize_lang("100000000", "zh"), "yi yi");
}

#[test]
fn test_zh_money() {
    assert_eq!(tn_normalize_lang("\u{00a5}100", "zh"), "yi bai yuan");
    assert_eq!(tn_normalize_lang("$50", "zh"), "wu shi meiyuan");
}

#[test]
fn test_zh_time() {
    assert_eq!(tn_normalize_lang("14:30", "zh"), "shi si dian san shi fen");
    assert_eq!(tn_normalize_lang("12:00", "zh"), "shi er dian zheng");
}

#[test]
fn test_zh_sentence() {
    assert_eq!(
        tn_normalize_sentence_lang("price is $50 ok", "zh"),
        "price is wu shi meiyuan ok"
    );
}

// ── Hindi ────────────────────────────────────────────────────────────

#[test]
fn test_hi_cardinal() {
    assert_eq!(tn_normalize_lang("123", "hi"), "ek sau teis");
    assert_eq!(tn_normalize_lang("100000", "hi"), "ek lakh");
    assert_eq!(tn_normalize_lang("10000000", "hi"), "ek crore");
}

#[test]
fn test_hi_money() {
    assert_eq!(tn_normalize_lang("\u{20b9}100", "hi"), "ek sau rupaye");
}

#[test]
fn test_hi_time() {
    assert_eq!(
        tn_normalize_lang("14:30", "hi"),
        "chaudah baj kar tees minat"
    );
}

#[test]
fn test_hi_sentence() {
    assert_eq!(
        tn_normalize_sentence_lang("price 100 rupees", "hi"),
        "price ek sau rupees"
    );
}

// ── Japanese ─────────────────────────────────────────────────────────

#[test]
fn test_ja_cardinal() {
    assert_eq!(tn_normalize_lang("123", "ja"), "hyaku ni juu san");
    assert_eq!(tn_normalize_lang("10000", "ja"), "ichi man");
    assert_eq!(tn_normalize_lang("300", "ja"), "sanbyaku");
}

#[test]
fn test_ja_money() {
    assert_eq!(tn_normalize_lang("\u{00a5}100", "ja"), "hyaku en");
    assert_eq!(tn_normalize_lang("$5", "ja"), "go doru");
}

#[test]
fn test_ja_time() {
    assert_eq!(tn_normalize_lang("14:30", "ja"), "juu yo ji san juppun");
    assert_eq!(tn_normalize_lang("9:00", "ja"), "ku ji");
}

#[test]
fn test_ja_sentence() {
    assert_eq!(
        tn_normalize_sentence_lang("price is $5 ok", "ja"),
        "price is go doru ok"
    );
}

// ── Cross-language dispatch ──────────────────────────────────────────

#[test]
fn test_same_input_different_languages() {
    // "123" should produce different output per language
    let results: Vec<(&str, String)> = ["en", "fr", "es", "de", "zh", "hi", "ja"]
        .iter()
        .map(|lang| (*lang, tn_normalize_lang("123", lang)))
        .collect();

    assert_eq!(results[0].1, "one hundred and twenty three"); // en
    assert_eq!(results[1].1, "cent vingt-trois"); // fr
    assert_eq!(results[2].1, "ciento veintitres"); // es
    assert_eq!(results[3].1, "einhundertdreiundzwanzig"); // de
    assert_eq!(results[4].1, "yi bai er shi san"); // zh
    assert_eq!(results[5].1, "ek sau teis"); // hi
    assert_eq!(results[6].1, "hyaku ni juu san"); // ja
}

#[test]
fn test_unknown_lang_falls_back_to_english() {
    assert_eq!(
        tn_normalize_lang("123", "xx"),
        "one hundred and twenty three"
    );
    assert_eq!(tn_normalize_lang("123", ""), "one hundred and twenty three");
}

#[test]
fn test_sentence_lang_passthrough() {
    // Non-normalizable text should pass through unchanged for all languages
    for lang in &["fr", "es", "de", "zh", "hi", "ja"] {
        assert_eq!(
            tn_normalize_sentence_lang("hello world", lang),
            "hello world",
            "passthrough failed for lang={}",
            lang
        );
    }
}

#[test]
fn test_sentence_lang_empty() {
    for lang in &["fr", "es", "de", "zh", "hi", "ja"] {
        assert_eq!(
            tn_normalize_sentence_lang("", lang),
            "",
            "empty failed for lang={}",
            lang
        );
    }
}
