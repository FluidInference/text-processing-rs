//! WebAssembly exports for JavaScript interop.

use wasm_bindgen::prelude::*;

#[cfg(feature = "fst-engine")]
use js_sys::{Array, Object, Reflect};

use crate::{
    custom_rules, normalize, normalize_sentence, normalize_sentence_lang,
    normalize_sentence_with_options, normalize_with_lang, normalize_with_options, tn_normalize,
    tn_normalize_lang, tn_normalize_sentence, tn_normalize_sentence_lang,
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

/// Compiled-FST TN with source-span alignment.
///
/// Returns `null` unless the build enables both `wasm` and `fst-engine`, or
/// when `lang` is unsupported. Offsets are half-open UTF-8 byte offsets.
#[wasm_bindgen(js_name = tnFstNormalizeAligned)]
pub fn tn_fst_normalize_aligned_js(input: &str, lang: &str) -> JsValue {
    #[cfg(not(feature = "fst-engine"))]
    {
        let _ = (input, lang);
        JsValue::NULL
    }

    #[cfg(feature = "fst-engine")]
    {
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
