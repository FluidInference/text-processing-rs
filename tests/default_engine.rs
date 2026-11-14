//! With `fst-engine`, the default entry points run NeMo's grammars, and the
//! rule-based taggers take over only for what NeMo doesn't cover.
#![cfg(feature = "fst-engine")]

use std::sync::Mutex;
use text_processing_rs::{
    custom_rules, fst, normalize, normalize_sentence, normalize_sentence_lang,
    normalize_sentence_lang_with_options, normalize_sentence_with_options, normalize_with_lang,
    set_rule_based_engine, tn_normalize, tn_normalize_lang, tn_normalize_sentence,
    tn_normalize_sentence_lang, NormalizeOptions,
};

/// Custom rules and the engine switch are process-wide.
static GLOBAL: Mutex<()> = Mutex::new(());

#[test]
fn itn_entry_points_run_nemo() {
    let _lock = GLOBAL.lock().unwrap();
    for (input, expected) in [
        // NeMo leaves "no one" and "three to five" alone; the taggers read numbers.
        (
            "No one knows, but I paid twenty five dollars for it.",
            "No one knows, but I paid $25 for it.",
        ),
        (
            "We need three to five people.",
            "We need three to five people.",
        ),
        (
            "Call me at five five five one two three four five six seven.",
            "Call me at 555-123-4567.",
        ),
        // NeMo's ITN expects unpunctuated input; the sentence keeps its own punctuation,
        // and one period ends `p.m.` at the end of a sentence.
        (
            "the meeting is at three pm.",
            "the meeting is at 03:00 p.m.",
        ),
        ("I said no, then twenty one...", "I said no, then 21..."),
    ] {
        assert_eq!(normalize_sentence(input), expected, "{input}");
        assert_eq!(normalize_sentence_lang(input, "en"), expected, "{input}");
        assert_eq!(normalize_with_lang(input, "en"), expected, "{input}");
        assert_eq!(normalize(input), expected, "{input}");
    }
    for (lang, input) in [
        ("de", "ich habe zwei hundert euro"),
        ("es", "tengo doscientos cincuenta y un euros"),
        ("fr", "j'ai vingt et un ans"),
        ("hi", "मेरे पास बीस रुपये हैं"),
        ("ja", "二十一"),
        ("zh", "一百二十三"),
    ] {
        let nemo = fst::inverse_normalize_lang(input, lang).unwrap();
        assert_eq!(
            normalize_sentence_lang(input, lang),
            nemo,
            "{lang}: {input}"
        );
        assert_eq!(normalize_with_lang(input, lang), nemo, "{lang}: {input}");
    }
    // Codes without NeMo ITN grammars read as English, as on the rule-based path.
    assert_eq!(normalize_sentence_lang("twenty one", "xx"), "21");
}

#[test]
fn tn_entry_points_run_nemo() {
    let _lock = GLOBAL.lock().unwrap();
    let input = "I have 5 in the box.";
    assert_eq!(tn_normalize_sentence(input), "I have five in the box.");
    assert_eq!(tn_normalize(input), "I have five in the box.");
    for lang in ["en", "de", "es", "fr", "hi", "it", "ja", "zh"] {
        let input = "123";
        let nemo = fst::normalize_lang(input, lang).unwrap();
        assert_eq!(tn_normalize_lang(input, lang), nemo, "{lang}");
        assert_eq!(tn_normalize_sentence_lang(input, lang), nemo, "{lang}");
    }
}

#[test]
fn itn_options_nemo_lacks_use_the_taggers() {
    let _lock = GLOBAL.lock().unwrap();
    let input = "United seven eighty eight";
    // NeMo reads this as 7808; the aviation option concatenates.
    assert_eq!(normalize_sentence(input), "United 7808");
    let aviation = NormalizeOptions::new().with_concat_compound_numbers(true);
    assert_eq!(
        normalize_sentence_with_options(input, aviation),
        "United 788"
    );
    assert_eq!(
        normalize_sentence_lang_with_options(input, "en", aviation),
        "United 788"
    );
    // max_span_tokens alone doesn't change the engine.
    let span = NormalizeOptions::new().with_max_span_tokens(4);
    assert_eq!(normalize_sentence_with_options(input, span), "United 7808");
}

#[test]
fn custom_rules_apply_ahead_of_nemo() {
    let _lock = GLOBAL.lock().unwrap();
    custom_rules::clear_rules();
    custom_rules::add_rule("gee pee tee", "GPT");
    custom_rules::add_rule("acme corp", "ACME Corp.");
    assert_eq!(
        normalize_sentence("I use Gee pee tee at acme corp, for twenty one dollars."),
        "I use GPT at ACME Corp., for $21."
    );
    assert_eq!(normalize("gee pee tee"), "GPT");
    custom_rules::clear_rules();
    assert_eq!(normalize_sentence("I use gee pee tee"), "I use gee pee tee");
}

#[test]
fn the_switch_runs_the_taggers() {
    let _lock = GLOBAL.lock().unwrap();
    assert_eq!(normalize("five"), "five");

    set_rule_based_engine(true);
    assert_eq!(normalize("five"), "5");
    assert_eq!(
        tn_normalize_sentence("I have 5 in the box."),
        "I have five in the box."
    );
    set_rule_based_engine(false);
    assert_eq!(normalize("five"), "five");
}
