//! Cardinal TN tagger.
//!
//! Converts written cardinal numbers to spoken form (NeMo conventions):
//! - "123" → "one hundred and twenty three" (British "and")
//! - "-42" → "minus forty two"
//! - "1,000" → "one thousand"
//! - "13000" → "one three zero zero zero" (unformatted >4 digits → digits)
//! - "004" → "zero zero four" (leading zero → digits)

use super::{number_to_words_and, spell_digits};

/// Parse a written cardinal number to spoken words.
///
/// Pure digit strings with an optional leading minus and comma grouping.
/// An *unformatted* (comma-less) integer that is longer than four digits or
/// carries a leading zero is read digit-by-digit, matching NeMo.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let (is_negative, digits_part) = if let Some(rest) = trimmed.strip_prefix('-') {
        (true, rest)
    } else {
        (false, trimmed)
    };

    // Must be digits (with optional commas), and contain at least one digit.
    if !digits_part.chars().all(|c| c.is_ascii_digit() || c == ',') {
        return None;
    }
    if !digits_part.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }

    let has_comma = digits_part.contains(',');
    let clean: String = digits_part.chars().filter(|c| *c != ',').collect();

    // Digit-string rule: an unformatted, non-negative integer that is longer
    // than four digits or has a leading zero reads digit-by-digit.
    if !is_negative && !has_comma {
        let leading_zero = clean.len() > 1 && clean.starts_with('0');
        if leading_zero || clean.len() > 4 {
            return Some(spell_digits(&clean));
        }
    }

    let n: u128 = clean.parse().ok()?;

    if is_negative {
        Some(format!("minus {}", number_to_words_and(n)))
    } else {
        Some(number_to_words_and(n))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(parse("0"), Some("zero".to_string()));
        assert_eq!(parse("1"), Some("one".to_string()));
        assert_eq!(parse("21"), Some("twenty one".to_string()));
        assert_eq!(parse("100"), Some("one hundred".to_string()));
        assert_eq!(
            parse("123"),
            Some("one hundred and twenty three".to_string())
        );
    }

    #[test]
    fn test_and_only_in_units_group() {
        assert_eq!(parse("9000"), Some("nine thousand".to_string()));
        assert_eq!(
            parse("123,000"),
            Some("one hundred twenty three thousand".to_string())
        );
        assert_eq!(
            parse("123,000,012"),
            Some("one hundred twenty three million twelve".to_string())
        );
    }

    #[test]
    fn test_digit_string() {
        assert_eq!(parse("13000"), Some("one three zero zero zero".to_string()));
        assert_eq!(
            parse("123000"),
            Some("one two three zero zero zero".to_string())
        );
        assert_eq!(parse("004"), Some("zero zero four".to_string()));
    }

    #[test]
    fn test_commas() {
        assert_eq!(parse("1,000"), Some("one thousand".to_string()));
        assert_eq!(parse("1,000,000"), Some("one million".to_string()));
        assert_eq!(parse("13,000"), Some("thirteen thousand".to_string()));
    }

    #[test]
    fn test_large_u128() {
        assert_eq!(
            parse("124,444,234,854,823,834,553"),
            Some("one hundred twenty four quintillion four hundred forty four quadrillion two hundred thirty four trillion eight hundred fifty four billion eight hundred twenty three million eight hundred thirty four thousand five hundred and fifty three".to_string())
        );
    }

    #[test]
    fn test_negative() {
        assert_eq!(parse("-42"), Some("minus forty two".to_string()));
        assert_eq!(parse("-1000"), Some("minus one thousand".to_string()));
    }

    #[test]
    fn test_non_numbers() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("12abc"), None);
        assert_eq!(parse(""), None);
    }
}
