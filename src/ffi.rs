//! C FFI bindings for Swift/ObjC interop.

use std::ffi::{c_char, CStr, CString};
use std::ptr;

use crate::{custom_rules, normalize, normalize_sentence, normalize_sentence_with_max_span};

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

    if custom_rules::remove_rule(spoken_str) { 1 } else { 0 }
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
    static VERSION: &[u8] = b"0.1.0\0";
    VERSION.as_ptr() as *const c_char
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
}
