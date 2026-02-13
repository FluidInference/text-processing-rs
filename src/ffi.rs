//! C FFI bindings for Swift/ObjC interop.

use std::ffi::{c_char, CStr, CString};
use std::ptr;

use crate::normalize;

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

/// Free a string allocated by nemo_normalize.
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
