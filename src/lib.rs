//! # NeMo-text-processing-rs
//!
//! Rust port of NVIDIA NeMo Text Processing for Inverse Text Normalization.
//!
//! Converts spoken-form text to written form:
//! - "two hundred thirty two" → "232"
//! - "five dollars and fifty cents" → "$5.50"
//! - "january fifth twenty twenty five" → "January 5, 2025"
//!
//! ## Usage
//!
//! ```
//! use nemo_text_processing::normalize;
//!
//! let result = normalize("two hundred");
//! assert_eq!(result, "200");
//! ```

pub mod taggers;

#[cfg(feature = "ffi")]
pub mod ffi;

use taggers::{cardinal, date, decimal, electronic, measure, money, ordinal, telephone, time, whitelist, word};

/// Normalize spoken-form text to written form.
///
/// Tries taggers in order of specificity (most specific first).
/// Returns original text if no tagger matches.
pub fn normalize(input: &str) -> String {
    let input = input.trim();

    // Apply whitelist replacements first (abbreviations, special terms)
    if let Some(result) = whitelist::parse(input) {
        return result;
    }

    // Try word patterns (spelled letters + numbers, numbers with punctuation)
    if let Some(result) = word::parse(input) {
        return result;
    }

    // Try time expressions (before telephone to avoid "two thirty" → alphanumeric)
    if let Some(result) = time::parse(input) {
        return result;
    }

    // Try date expressions (before telephone to avoid "nineteen ninety four" → alphanumeric)
    if let Some(result) = date::parse(input) {
        return result;
    }

    // Try money (contains number + currency) - before telephone
    if let Some(result) = money::parse(input) {
        return result;
    }

    // Try measurements (contains number + unit) - before telephone
    if let Some(result) = measure::parse(input) {
        return result;
    }

    // Try decimal numbers (before telephone to catch "sixty point two")
    if let Some(result) = decimal::parse(input) {
        return result;
    }

    // Try telephone/IP numbers (before electronic to catch IP addresses)
    if let Some(result) = telephone::parse(input) {
        return result;
    }

    // Try electronic addresses (emails, URLs)
    if let Some(result) = electronic::parse(input) {
        return result;
    }

    // Try decimal numbers
    if let Some(result) = decimal::parse(input) {
        return result;
    }

    // Try ordinal numbers
    if let Some(result) = ordinal::parse(input) {
        return result;
    }

    // Try cardinal number
    if let Some(num) = cardinal::parse(input) {
        return num;
    }

    // No match - return original
    input.to_string()
}

/// Normalize with language selection (future use).
pub fn normalize_with_lang(input: &str, _lang: &str) -> String {
    // TODO: Language-specific taggers
    normalize(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_cardinal() {
        assert_eq!(normalize("one"), "1");
        assert_eq!(normalize("twenty one"), "21");
        assert_eq!(normalize("one hundred"), "100");
    }

    #[test]
    fn test_basic_money() {
        assert_eq!(normalize("five dollars"), "$5");
    }

    #[test]
    fn test_passthrough() {
        assert_eq!(normalize("hello world"), "hello world");
    }
}
