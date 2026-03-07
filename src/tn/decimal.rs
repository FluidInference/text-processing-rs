//! Decimal TN tagger.
//!
//! Converts written decimal numbers to spoken form:
//! - "3.14" → "three point one four"
//! - "0.5" → "zero point five"
//! - "1.5 billion" → "one point five billion"

use super::{number_to_words, spell_digits};

/// Scale suffixes we recognize after a decimal number.
const QUANTITY_SUFFIXES: &[&str] = &[
    "billion", "million", "trillion", "quadrillion", "quintillion", "thousand",
];

/// Parse a written decimal number to spoken form.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Check for quantity suffix: "1.5 billion"
    let (number_part, suffix) = extract_suffix(trimmed);

    // Must contain a decimal point
    if !number_part.contains('.') {
        return None;
    }

    let parts: Vec<&str> = number_part.splitn(2, '.').collect();
    if parts.len() != 2 {
        return None;
    }

    let int_str = parts[0];
    let frac_str = parts[1];

    // Both parts should be digits (int part may be empty → "0", may have leading minus)
    let (is_negative, int_digits) = if let Some(rest) = int_str.strip_prefix('-') {
        (true, rest)
    } else {
        (false, int_str)
    };

    if !int_digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    if frac_str.is_empty() || !frac_str.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    let int_val: i64 = if int_digits.is_empty() {
        0
    } else {
        int_digits.parse().ok()?
    };

    let int_words = number_to_words(int_val);
    let frac_words = spell_digits(frac_str);

    let mut result = if is_negative {
        format!("minus {} point {}", int_words, frac_words)
    } else {
        format!("{} point {}", int_words, frac_words)
    };

    if let Some(suf) = suffix {
        result.push(' ');
        result.push_str(suf);
    }

    Some(result)
}

/// Extract a quantity suffix from the end if present.
fn extract_suffix(input: &str) -> (&str, Option<&str>) {
    for &suf in QUANTITY_SUFFIXES {
        if let Some(before) = input.strip_suffix(suf) {
            let before = before.trim_end();
            if !before.is_empty() {
                return (before, Some(suf));
            }
        }
    }
    (input, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_decimal() {
        assert_eq!(parse("3.14"), Some("three point one four".to_string()));
        assert_eq!(parse("0.5"), Some("zero point five".to_string()));
        assert_eq!(parse("1.0"), Some("one point zero".to_string()));
    }

    #[test]
    fn test_with_quantity() {
        assert_eq!(
            parse("1.5 billion"),
            Some("one point five billion".to_string())
        );
        assert_eq!(
            parse("4.85 billion"),
            Some("four point eight five billion".to_string())
        );
    }

    #[test]
    fn test_negative_decimal() {
        assert_eq!(
            parse("-3.14"),
            Some("minus three point one four".to_string())
        );
    }

    #[test]
    fn test_non_decimal() {
        assert_eq!(parse("123"), None);
        assert_eq!(parse("hello"), None);
        assert_eq!(parse(""), None);
    }
}
