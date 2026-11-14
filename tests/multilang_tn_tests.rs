//! Multi-language text normalization integration tests.
//!
//! Tests the language dispatch API: tn_normalize_lang() and tn_normalize_sentence_lang().
//! Ensures all 6 languages are wired up and producing correct output through the public API.
// These tests pin the rule-based taggers' output. With `fst-engine` the default
// entry points run NeMo's grammars instead (tests/default_engine.rs).
#![cfg(not(feature = "fst-engine"))]

use std::fs;
use std::path::Path;
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

// ── Sentence spans (#96) ─────────────────────────────────────────────

#[test]
fn test_de_sentence_keeps_words_around_spans() {
    // A span match used to swallow the words around it and glue the date's
    // digits onto the price ("dreihundertzehn millionen ... euro").
    assert_eq!(
        tn_normalize_sentence_lang("Am 3.10.2025 kostete es 12,50 € für 2 Stück.", "de"),
        "Am dritter oktober zweitausendfuenfundzwanzig kostete es zwoelf euro und fuenfzig cent für zwei Stück."
    );
    assert_eq!(
        tn_normalize_sentence_lang("Es kostet 12,50 €.", "de"),
        "Es kostet zwoelf euro und fuenfzig cent."
    );
    assert_eq!(
        tn_normalize_sentence_lang("Sie kam am 3. Oktober 2025 nach Hause.", "de"),
        "Sie kam am dritter oktober zweitausendfuenfundzwanzig nach Hause."
    );
    assert_eq!(
        tn_normalize_sentence_lang("Wir treffen uns um 17:30 Uhr am Bahnhof.", "de"),
        "Wir treffen uns um siebzehn uhr dreissig am Bahnhof."
    );
    // The period after a money amount is the sentence's.
    assert_eq!(
        tn_normalize_sentence_lang("Es kostet $5.", "de"),
        "Es kostet fuenf dollar."
    );
}

#[test]
fn test_fr_es_sentence_keeps_words_around_spans() {
    assert_eq!(
        tn_normalize_sentence_lang("Il coûte 12,50 € aujourd'hui.", "fr"),
        "Il coûte douze euros et cinquante centimes aujourd'hui."
    );
    assert_eq!(
        tn_normalize_sentence_lang("Le 3 octobre 2025 il est venu à la maison.", "fr"),
        "Le trois octobre deux mille vingt-cinq il est venu à la maison."
    );
    assert_eq!(
        tn_normalize_sentence_lang("Cuesta 12,50 € hoy.", "es"),
        "Cuesta doce euros con cincuenta centimos hoy."
    );
    assert_eq!(
        tn_normalize_sentence_lang("El 3 de octubre de 2025 llegó a casa.", "es"),
        "El tres de octubre de dos mil veinticinco llegó a casa."
    );
    // The decimal comma survives a sentence-final period.
    assert_eq!(
        tn_normalize_sentence_lang("Hoy vimos 1,26.", "es"),
        "Hoy vimos uno coma dos seis."
    );
}

/// Every written form a TN tagger accepts alone (the written side of the ITN
/// data), placed inside a sentence and at its end, keeps the words around it
/// and the final period.
#[test]
fn test_sentence_spans_keep_surrounding_words() {
    let frames = [
        ("en", "Today we saw", "for them"),
        ("de", "Heute sahen wir", "bei ihnen"),
        ("fr", "Hier nous avons vu", "chez eux"),
        ("es", "Hoy vimos", "con ellos"),
    ];
    let mut failures = Vec::new();
    for (lang, lead, trail) in frames {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data")
            .join(lang);
        for entry in fs::read_dir(&dir).expect("data dir") {
            let path = entry.unwrap().path();
            let name = path.file_stem().unwrap().to_string_lossy().to_string();
            if name.contains("cased") || name == "word" || name == "whitelist" {
                continue;
            }
            let data = fs::read_to_string(&path).expect("data file");
            for line in data.lines().filter(|line| !line.starts_with('#')) {
                let Some((_, written)) = line.split_once('~') else {
                    continue;
                };
                let x = written.trim();
                if x.ends_with('.') || tn_normalize_lang(x, lang) == x {
                    continue;
                }
                let mut sentences = vec![format!("{lead} {x} {trail}.")];
                // NeMo reads a German number before the final period as an
                // ordinal ("100." -> "hundertste"), so only check it mid-sentence.
                if !(lang == "de" && x.chars().all(|c| c.is_ascii_digit())) {
                    sentences.push(format!("{lead} {x}."));
                }
                for sentence in sentences {
                    let out = tn_normalize_sentence_lang(&sentence, lang);
                    let tail = if sentence.ends_with(&format!("{trail}.")) {
                        format!(" {trail}.")
                    } else {
                        ".".to_string()
                    };
                    if !out.starts_with(&format!("{lead} ")) || !out.ends_with(&tail) {
                        failures.push(format!("[{lang}] {sentence:?} -> {out:?}"));
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn test_large_numbers_do_not_panic() {
    // de, es and ja ran out of scale words and indexed past their tables.
    for lang in ["en", "fr", "es", "de", "zh", "hi", "ja"] {
        for n in [
            "1000000001",
            "1000000000000000",
            "18446744073709551615",
            "99999999999999999999999",
        ] {
            for input in [
                n.to_string(),
                format!("{n} km"),
                format!("${n}"),
                format!("{n} €"),
            ] {
                tn_normalize_lang(&input, lang);
                tn_normalize_sentence_lang(&format!("Wir sahen {input} heute."), lang);
            }
        }
    }
}
