//! Decimal TN tagger for French.
//!
//! Converts written decimal numbers to spoken French:
//! - "3,14" → "trois virgule un quatre"
//! - "3.14" → "trois virgule un quatre"
//! - "0,5" → "zero virgule cinq"
//! - "1,5 milliard" → "un virgule cinq milliard"

use super::{number_to_words, spell_digits};

/// French quantity suffixes recognized after a decimal number.
const QUANTITY_SUFFIXES: &[&str] = &[
    "billiard",
    "billion",
    "milliard",
    "milliards",
    "million",
    "millions",
    "mille",
];

/// Parse a written decimal number to spoken French.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Check for quantity suffix: "1,5 milliard"
    let (number_part, suffix) = extract_suffix(trimmed);

    // French uses comma as decimal separator, but also accept period
    let sep = if number_part.contains(',') && !number_part.contains('.') {
        ','
    } else if number_part.contains('.') {
        '.'
    } else {
        return None;
    };

    let parts: Vec<&str> = number_part.splitn(2, sep).collect();
    if parts.len() != 2 {
        return None;
    }

    let int_str = parts[0];
    let frac_str = parts[1];

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
        format!("moins {} virgule {}", int_words, frac_words)
    } else {
        format!("{} virgule {}", int_words, frac_words)
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
    fn test_comma_decimal() {
        assert_eq!(parse("3,14"), Some("trois virgule un quatre".to_string()));
        assert_eq!(parse("0,5"), Some("zero virgule cinq".to_string()));
    }

    #[test]
    fn test_period_decimal() {
        assert_eq!(parse("3.14"), Some("trois virgule un quatre".to_string()));
    }

    #[test]
    fn test_negative() {
        assert_eq!(
            parse("-3,14"),
            Some("moins trois virgule un quatre".to_string())
        );
    }

    #[test]
    fn test_with_quantity() {
        assert_eq!(
            parse("1,5 milliard"),
            Some("un virgule cinq milliard".to_string())
        );
        assert_eq!(
            parse("4,85 millions"),
            Some("quatre virgule huit cinq millions".to_string())
        );
    }

    #[test]
    fn test_non_decimal() {
        assert_eq!(parse("123"), None);
        assert_eq!(parse("hello"), None);
    }
}
