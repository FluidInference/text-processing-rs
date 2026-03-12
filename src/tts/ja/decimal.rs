//! Decimal TN tagger for Japanese (romaji output).
//!
//! Converts written decimal numbers to spoken Japanese in romaji:
//! - "3.14" → "san ten ichi yon"
//! - "0.5" → "zero ten go"
//! - "-2.7" → "mainasu ni ten nana"

use super::{number_to_words, spell_digits};

/// Japanese quantity suffixes recognized after a decimal number.
/// oku (億) = hundred million, man (万) = ten thousand
const QUANTITY_SUFFIXES: &[&str] = &["oku", "man"];

/// Parse a written decimal number to spoken Japanese in romaji.
///
/// Uses "ten" (点) as the decimal point word.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Check for quantity suffix: "1.5 man"
    let (number_part, suffix) = extract_suffix(trimmed);

    // Accept both period and comma as decimal separator
    let sep = if number_part.contains('.') {
        '.'
    } else if number_part.contains(',') {
        ','
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
        format!("mainasu {} ten {}", int_words, frac_words)
    } else {
        format!("{} ten {}", int_words, frac_words)
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
        assert_eq!(parse("3.14"), Some("san ten ichi yon".to_string()));
        assert_eq!(parse("0.5"), Some("zero ten go".to_string()));
        assert_eq!(parse("1.0"), Some("ichi ten zero".to_string()));
    }

    #[test]
    fn test_negative_decimal() {
        assert_eq!(
            parse("-3.14"),
            Some("mainasu san ten ichi yon".to_string())
        );
        assert_eq!(parse("-0.5"), Some("mainasu zero ten go".to_string()));
    }

    #[test]
    fn test_comma_decimal() {
        assert_eq!(parse("3,14"), Some("san ten ichi yon".to_string()));
    }

    #[test]
    fn test_with_quantity() {
        assert_eq!(
            parse("1.5 man"),
            Some("ichi ten go man".to_string())
        );
        assert_eq!(
            parse("4.85 oku"),
            Some("yon ten hachi go oku".to_string())
        );
    }

    #[test]
    fn test_non_decimal() {
        assert_eq!(parse("123"), None);
        assert_eq!(parse("hello"), None);
        assert_eq!(parse(""), None);
    }
}
