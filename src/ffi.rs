//! C FFI bindings for Swift/ObjC interop.

use std::ffi::{c_char, CStr, CString};
use std::ptr;

use crate::{
    custom_rules, normalize, normalize_aviation, normalize_sentence, normalize_sentence_aviation,
    normalize_sentence_aviation_with_max_span, normalize_sentence_with_max_span, tn_normalize,
    tn_normalize_lang, tn_normalize_sentence, tn_normalize_sentence_lang,
    tn_normalize_sentence_with_max_span, tn_normalize_sentence_with_max_span_lang,
};

/// Normalize spoken-form text to written form.
///
/// # Safety
/// - `input` must be a valid null-terminated UTF-8 string
/// - Returns a newly allocated string that must be freed with `nemo_free_string`
#[no_mangle]
pub unsafe extern "C" fn nemo_normalize(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return ptr::null_mut();
    }

    let c_str = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let result = normalize(c_str);

    match CString::new(result) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Normalize a full sentence, replacing spoken-form spans with written form.
///
/// Unlike `nemo_normalize` which expects the entire input to be a single expression,
/// this scans for normalizable spans within a larger sentence.
///
/// # Safety
/// - `input` must be a valid null-terminated UTF-8 string
/// - Returns a newly allocated string that must be freed with `nemo_free_string`
#[no_mangle]
pub unsafe extern "C" fn nemo_normalize_sentence(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return ptr::null_mut();
    }

    let c_str = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let result = normalize_sentence(c_str);

    match CString::new(result) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Normalize a full sentence with a configurable max span size.
///
/// `max_span_tokens` controls the maximum number of consecutive tokens
/// considered as a single normalizable expression (default is 16).
///
/// # Safety
/// - `input` must be a valid null-terminated UTF-8 string
/// - Returns a newly allocated string that must be freed with `nemo_free_string`
#[no_mangle]
pub unsafe extern "C" fn nemo_normalize_sentence_with_max_span(
    input: *const c_char,
    max_span_tokens: u32,
) -> *mut c_char {
    if input.is_null() {
        return ptr::null_mut();
    }

    let c_str = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let result = normalize_sentence_with_max_span(c_str, max_span_tokens as usize);

    match CString::new(result) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Aviation-flavoured single-input normalize.
///
/// Layered on top of [`nemo_normalize`]: tries `cardinal::parse_aviation`
/// first so flight-number / call-sign phrases like `"seven eighty eight"`
/// resolve to `"788"`, then falls back to the regular dispatch.
///
/// # Safety
/// - `input` must be a valid null-terminated UTF-8 string
/// - Returns a newly allocated string that must be freed with `nemo_free_string`
#[no_mangle]
pub unsafe extern "C" fn nemo_normalize_aviation(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return ptr::null_mut();
    }

    let c_str = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let result = normalize_aviation(c_str);

    match CString::new(result) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Aviation-flavoured sentence normalize.
///
/// Sentence-mode equivalent of [`nemo_normalize_aviation`]. Aviation cardinal
/// runs at priority 89 (above date / time, below money / measure), so
/// flight-number-style spans win without disturbing money / measure / decimal.
///
/// # Safety
/// - `input` must be a valid null-terminated UTF-8 string
/// - Returns a newly allocated string that must be freed with `nemo_free_string`
#[no_mangle]
pub unsafe extern "C" fn nemo_normalize_sentence_aviation(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return ptr::null_mut();
    }

    let c_str = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let result = normalize_sentence_aviation(c_str);

    match CString::new(result) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Aviation sentence normalize with a configurable max span size.
///
/// # Safety
/// - `input` must be a valid null-terminated UTF-8 string
/// - Returns a newly allocated string that must be freed with `nemo_free_string`
#[no_mangle]
pub unsafe extern "C" fn nemo_normalize_sentence_aviation_with_max_span(
    input: *const c_char,
    max_span_tokens: u32,
) -> *mut c_char {
    if input.is_null() {
        return ptr::null_mut();
    }

    let c_str = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let result = normalize_sentence_aviation_with_max_span(c_str, max_span_tokens as usize);

    match CString::new(result) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Free a string allocated by nemo_normalize or nemo_normalize_sentence.
///
/// # Safety
/// - `s` must be a pointer returned by `nemo_normalize`, or null
/// - Must not be called twice on the same pointer
#[no_mangle]
pub unsafe extern "C" fn nemo_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

/// Add a custom spoken→written normalization rule.
///
/// Custom rules have the highest priority and are checked before all built-in taggers.
/// If a rule with the same spoken form exists, it is replaced.
///
/// # Safety
/// - `spoken` and `written` must be valid null-terminated UTF-8 strings
#[no_mangle]
pub unsafe extern "C" fn nemo_add_rule(spoken: *const c_char, written: *const c_char) {
    if spoken.is_null() || written.is_null() {
        return;
    }

    let spoken_str = match CStr::from_ptr(spoken).to_str() {
        Ok(s) => s,
        Err(_) => return,
    };
    let written_str = match CStr::from_ptr(written).to_str() {
        Ok(s) => s,
        Err(_) => return,
    };

    custom_rules::add_rule(spoken_str, written_str);
}

/// Remove a custom normalization rule by its spoken form.
///
/// # Safety
/// - `spoken` must be a valid null-terminated UTF-8 string
/// - Returns 1 if the rule was found and removed, 0 otherwise
#[no_mangle]
pub unsafe extern "C" fn nemo_remove_rule(spoken: *const c_char) -> i32 {
    if spoken.is_null() {
        return 0;
    }

    let spoken_str = match CStr::from_ptr(spoken).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    if custom_rules::remove_rule(spoken_str) {
        1
    } else {
        0
    }
}

/// Clear all custom normalization rules.
#[no_mangle]
pub extern "C" fn nemo_clear_rules() {
    custom_rules::clear_rules();
}

/// Get the number of custom rules currently registered.
#[no_mangle]
pub extern "C" fn nemo_rule_count() -> u32 {
    custom_rules::rule_count() as u32
}

/// Get the library version.
///
/// # Safety
/// Returns a static string, do not free.
#[no_mangle]
pub extern "C" fn nemo_version() -> *const c_char {
    static VERSION: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();
    VERSION.as_ptr() as *const c_char
}

// ── Text Normalization (written → spoken) FFI ─────────────────────────

/// Normalize written-form text to spoken form (Text Normalization).
///
/// # Safety
/// - `input` must be a valid null-terminated UTF-8 string
/// - Returns a newly allocated string that must be freed with `nemo_free_string`
#[no_mangle]
pub unsafe extern "C" fn nemo_tn_normalize(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return ptr::null_mut();
    }

    let c_str = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let result = tn_normalize(c_str);

    match CString::new(result) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Normalize a full sentence from written to spoken form (TN).
///
/// # Safety
/// - `input` must be a valid null-terminated UTF-8 string
/// - Returns a newly allocated string that must be freed with `nemo_free_string`
#[no_mangle]
pub unsafe extern "C" fn nemo_tn_normalize_sentence(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return ptr::null_mut();
    }

    let c_str = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let result = tn_normalize_sentence(c_str);

    match CString::new(result) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Normalize a full sentence (TN) with a configurable max span size.
///
/// # Safety
/// - `input` must be a valid null-terminated UTF-8 string
/// - Returns a newly allocated string that must be freed with `nemo_free_string`
#[no_mangle]
pub unsafe extern "C" fn nemo_tn_normalize_sentence_with_max_span(
    input: *const c_char,
    max_span_tokens: u32,
) -> *mut c_char {
    if input.is_null() {
        return ptr::null_mut();
    }

    let c_str = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let result = tn_normalize_sentence_with_max_span(c_str, max_span_tokens as usize);

    match CString::new(result) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

// ── Language-aware TN FFI ──────────────────────────────────────────────

/// Normalize written-form text to spoken form for a specific language.
///
/// Supported language codes: "en", "fr", "es", "de", "zh", "hi", "ja".
/// Falls back to English for unrecognized codes.
///
/// # Safety
/// - `input` and `lang` must be valid null-terminated UTF-8 strings
/// - Returns a newly allocated string that must be freed with `nemo_free_string`
#[no_mangle]
pub unsafe extern "C" fn nemo_tn_normalize_lang(
    input: *const c_char,
    lang: *const c_char,
) -> *mut c_char {
    if input.is_null() || lang.is_null() {
        return ptr::null_mut();
    }

    let input_str = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let lang_str = match CStr::from_ptr(lang).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let result = tn_normalize_lang(input_str, lang_str);

    match CString::new(result) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Normalize a full sentence (TN) for a specific language.
///
/// # Safety
/// - `input` and `lang` must be valid null-terminated UTF-8 strings
/// - Returns a newly allocated string that must be freed with `nemo_free_string`
#[no_mangle]
pub unsafe extern "C" fn nemo_tn_normalize_sentence_lang(
    input: *const c_char,
    lang: *const c_char,
) -> *mut c_char {
    if input.is_null() || lang.is_null() {
        return ptr::null_mut();
    }

    let input_str = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let lang_str = match CStr::from_ptr(lang).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let result = tn_normalize_sentence_lang(input_str, lang_str);

    match CString::new(result) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Normalize a full sentence (TN) for a specific language with configurable max span.
///
/// # Safety
/// - `input` and `lang` must be valid null-terminated UTF-8 strings
/// - Returns a newly allocated string that must be freed with `nemo_free_string`
#[no_mangle]
pub unsafe extern "C" fn nemo_tn_normalize_sentence_with_max_span_lang(
    input: *const c_char,
    lang: *const c_char,
    max_span_tokens: u32,
) -> *mut c_char {
    if input.is_null() || lang.is_null() {
        return ptr::null_mut();
    }

    let input_str = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let lang_str = match CStr::from_ptr(lang).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let result =
        tn_normalize_sentence_with_max_span_lang(input_str, lang_str, max_span_tokens as usize);

    match CString::new(result) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_normalize() {
        unsafe {
            let input = CString::new("two hundred").unwrap();
            let result = nemo_normalize(input.as_ptr());
            assert!(!result.is_null());
            let result_str = CStr::from_ptr(result).to_str().unwrap();
            assert_eq!(result_str, "200");
            nemo_free_string(result);
        }
    }

    #[test]
    fn test_ffi_null_input() {
        unsafe {
            let result = nemo_normalize(ptr::null());
            assert!(result.is_null());
        }
    }

    #[test]
    fn test_ffi_normalize_aviation() {
        unsafe {
            let input = CString::new("seven eighty eight").unwrap();
            let result = nemo_normalize_aviation(input.as_ptr());
            assert!(!result.is_null());
            let result_str = CStr::from_ptr(result).to_str().unwrap();
            assert_eq!(result_str, "788");
            nemo_free_string(result);
        }
    }

    #[test]
    fn test_ffi_normalize_sentence_aviation() {
        unsafe {
            let input = CString::new("United seven eighty eight").unwrap();
            let result = nemo_normalize_sentence_aviation(input.as_ptr());
            assert!(!result.is_null());
            let result_str = CStr::from_ptr(result).to_str().unwrap();
            assert_eq!(result_str, "United 788");
            nemo_free_string(result);
        }
    }
}
