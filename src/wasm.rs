//! WebAssembly exports for JavaScript interop.

use wasm_bindgen::prelude::*;

#[cfg(feature = "fst-engine")]
use js_sys::{Array, Object, Reflect};

use crate::{
    custom_rules, normalize, normalize_sentence, normalize_sentence_lang,
    normalize_sentence_lang_with_options, normalize_sentence_with_options, normalize_with_lang,
    normalize_with_options, tn_normalize, tn_normalize_lang, tn_normalize_sentence,
    tn_normalize_sentence_lang, tn_normalize_sentence_lang_with_options,
    tn_normalize_sentence_with_max_span, tn_normalize_sentence_with_max_span_lang,
    NormalizeOptions,
};

/// Build [`NormalizeOptions`] from JS-friendly primitives.
///
/// `max_span_tokens == 0` is treated as "use library default" so JS callers
/// can pass `0` rather than dealing with optional values across the boundary.
fn js_options(
    concat_compound_numbers: bool,
    max_span_tokens: u32,
    disable_bare_second: bool,
) -> NormalizeOptions {
    NormalizeOptions {
        concat_compound_numbers,
        max_span_tokens: if max_span_tokens == 0 {
            None
        } else {
            Some(max_span_tokens as usize)
        },
        disable_bare_second,
        roman_enumerators: false,
        small_numbers_as_words: false,
    }
}

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

#[wasm_bindgen(js_name = normalizeSentenceLang)]
pub fn normalize_sentence_lang_js(input: &str, lang: &str) -> String {
    normalize_sentence_lang(input, lang)
}

/// Unified single-expression normalize. `concatCompoundNumbers=true` reads
/// consecutive number words as concatenation rather than addition, e.g.
/// `"thirty five sixty two"` → `"3562"`, `"seven eighty eight"` → `"788"`.
/// `disableBareSecond=true` blocks the bare word `"second"` from converting
/// to `"2nd"` (issue #22).
#[wasm_bindgen(js_name = normalizeWithOptions)]
pub fn normalize_with_options_js(
    input: &str,
    concat_compound_numbers: bool,
    disable_bare_second: bool,
) -> String {
    normalize_with_options(
        input,
        js_options(concat_compound_numbers, 0, disable_bare_second),
    )
}

/// Unified sentence normalize. `concatCompoundNumbers` mirrors the
/// single-expression flag; `maxSpanTokens == 0` means "use library default"
/// (16). `disableBareSecond=true` keeps phrases like `"give me a second"`
/// literal (issue #22).
#[wasm_bindgen(js_name = normalizeSentenceWithOptions)]
pub fn normalize_sentence_with_options_js(
    input: &str,
    concat_compound_numbers: bool,
    max_span_tokens: u32,
    disable_bare_second: bool,
) -> String {
    normalize_sentence_with_options(
        input,
        js_options(
            concat_compound_numbers,
            max_span_tokens,
            disable_bare_second,
        ),
    )
}

/// Sentence ITN for a specific language with options. `maxSpanTokens == 0`
/// means "use library default" (16). `smallNumbersAsWords=true` keeps a lone
/// "zero" to "twelve" as a word and reads "three to five" as a time only when
/// a.m. or p.m. follows, as NeMo does (English, issue #109).
#[wasm_bindgen(js_name = normalizeSentenceLangWithOptions)]
pub fn normalize_sentence_lang_with_options_js(
    input: &str,
    lang: &str,
    concat_compound_numbers: bool,
    max_span_tokens: u32,
    disable_bare_second: bool,
    small_numbers_as_words: bool,
) -> String {
    let options = js_options(
        concat_compound_numbers,
        max_span_tokens,
        disable_bare_second,
    )
    .with_small_numbers_as_words(small_numbers_as_words);
    normalize_sentence_lang_with_options(input, lang, options)
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

/// Sentence TN with options. `maxSpanTokens == 0` means "use library
/// default" (16). `romanEnumerators=true` reads English roman-numeral list
/// markers as numbers (`(ii)` → `(two)`) before the taggers run
/// (FluidAudio #972).
#[wasm_bindgen(js_name = tnNormalizeSentenceLangWithOptions)]
pub fn tn_normalize_sentence_lang_with_options_js(
    input: &str,
    lang: &str,
    max_span_tokens: u32,
    roman_enumerators: bool,
) -> String {
    let options =
        js_options(false, max_span_tokens, false).with_roman_enumerators(roman_enumerators);
    tn_normalize_sentence_lang_with_options(input, lang, options)
}

/// Compiled-FST TN with source-span alignment. Exported only by builds with the
/// `fst-engine` feature; the published npm package is built without it.
///
/// Returns `{ normalized, spans: [{ inputStart, inputEnd, original, normalized,
/// kind }] }`, or `null` when `lang` is unsupported or the grammar cannot
/// classify `input`. Offsets are half-open UTF-8 byte offsets into `input`.
#[cfg(feature = "fst-engine")]
#[wasm_bindgen(js_name = tnFstNormalizeAligned)]
pub fn tn_fst_normalize_aligned_js(input: &str, lang: &str) -> JsValue {
    let Some(alignment) = crate::fst::normalize_aligned(input, lang) else {
        return JsValue::NULL;
    };
    let result = Object::new();
    let spans = Array::new();
    for span in alignment.spans {
        let item = Object::new();
        set_js_property(
            &item,
            "inputStart",
            &JsValue::from_f64(span.input_start as f64),
        );
        set_js_property(&item, "inputEnd", &JsValue::from_f64(span.input_end as f64));
        set_js_property(&item, "original", &JsValue::from_str(&span.original));
        set_js_property(&item, "normalized", &JsValue::from_str(&span.normalized));
        set_js_property(&item, "kind", &JsValue::from_str(span.kind.as_str()));
        spans.push(&item);
    }
    set_js_property(
        &result,
        "normalized",
        &JsValue::from_str(&alignment.normalized),
    );
    set_js_property(&result, "spans", &spans);
    result.into()
}

#[cfg(feature = "fst-engine")]
fn set_js_property(object: &Object, name: &str, value: &JsValue) {
    let _ = Reflect::set(object, &JsValue::from_str(name), value);
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
