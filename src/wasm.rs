//! WebAssembly exports for JavaScript interop.

use wasm_bindgen::prelude::*;

use crate::{
    custom_rules, normalize, normalize_sentence, normalize_sentence_with_max_span,
    normalize_with_lang, tn_normalize, tn_normalize_lang, tn_normalize_sentence,
    tn_normalize_sentence_lang, tn_normalize_sentence_with_max_span,
    tn_normalize_sentence_with_max_span_lang,
};

/// Initialize panic hook for better error messages in browser devtools.
#[wasm_bindgen]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen(js_name = normalize)]
pub fn normalize_js(input: &str) -> String {
    normalize(input)
}

#[wasm_bindgen(js_name = normalizeWithLang)]
pub fn normalize_with_lang_js(input: &str, lang: &str) -> String {
    normalize_with_lang(input, lang)
}

#[wasm_bindgen(js_name = normalizeSentence)]
pub fn normalize_sentence_js(input: &str) -> String {
    normalize_sentence(input)
}

#[wasm_bindgen(js_name = normalizeSentenceWithMaxSpan)]
pub fn normalize_sentence_with_max_span_js(input: &str, max_span_tokens: u32) -> String {
    normalize_sentence_with_max_span(input, max_span_tokens as usize)
}

#[wasm_bindgen(js_name = tnNormalize)]
pub fn tn_normalize_js(input: &str) -> String {
    tn_normalize(input)
}

#[wasm_bindgen(js_name = tnNormalizeLang)]
pub fn tn_normalize_lang_js(input: &str, lang: &str) -> String {
    tn_normalize_lang(input, lang)
}

#[wasm_bindgen(js_name = tnNormalizeSentence)]
pub fn tn_normalize_sentence_js(input: &str) -> String {
    tn_normalize_sentence(input)
}

#[wasm_bindgen(js_name = tnNormalizeSentenceLang)]
pub fn tn_normalize_sentence_lang_js(input: &str, lang: &str) -> String {
    tn_normalize_sentence_lang(input, lang)
}

#[wasm_bindgen(js_name = tnNormalizeSentenceWithMaxSpan)]
pub fn tn_normalize_sentence_with_max_span_js(input: &str, max_span_tokens: u32) -> String {
    tn_normalize_sentence_with_max_span(input, max_span_tokens as usize)
}

#[wasm_bindgen(js_name = tnNormalizeSentenceWithMaxSpanLang)]
pub fn tn_normalize_sentence_with_max_span_lang_js(
    input: &str,
    lang: &str,
    max_span_tokens: u32,
) -> String {
    tn_normalize_sentence_with_max_span_lang(input, lang, max_span_tokens as usize)
}

#[wasm_bindgen(js_name = addRule)]
pub fn add_rule_js(spoken: &str, written: &str) {
    custom_rules::add_rule(spoken, written);
}

#[wasm_bindgen(js_name = removeRule)]
pub fn remove_rule_js(spoken: &str) -> bool {
    custom_rules::remove_rule(spoken)
}

#[wasm_bindgen(js_name = clearRules)]
pub fn clear_rules_js() {
    custom_rules::clear_rules();
}

#[wasm_bindgen(js_name = ruleCount)]
pub fn rule_count_js() -> u32 {
    custom_rules::rule_count() as u32
}
