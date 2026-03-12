//! Decimal TN tagger for Hindi (romanized).
//!
//! Converts written decimal numbers to spoken romanized Hindi:
//! - "3.14" → "teen dashmlav ek chaar"
//! - "0.5" → "shunya dashmlav paanch"
//! - "-2.7" → "rhin do dashmlav saat"

use super::{number_to_words, spell_digits};

/// Hindi quantity suffixes recognized after a decimal number.
/// crore, lakh, hazaar (thousand)
const QUANTITY_SUFFIXES: &[&str] = &["crore", "lakh", "hazaar"];

/// Parse a written decimal number to spoken romanized Hindi.
///
/// Uses "dashmlav" for the decimal point. The fractional part is spelled
/// digit by digit.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Check for quantity suffix: "1.5 lakh"
    let (number_part, suffix) = extract_suffix(trimmed);

    // Hindi typically uses period as decimal separator
    if !number_part.contains('.') {
        return None;
    }

    let parts: Vec<&str> = number_part.splitn(2, '.').collect();
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
        format!("rhin {} dashmlav {}", int_words, frac_words)
    } else {
        format!("{} dashmlav {}", int_words, frac_words)
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
        assert_eq!(parse("3.14"), Some("teen dashmlav ek chaar".to_string()));
        assert_eq!(parse("0.5"), Some("shunya dashmlav paanch".to_string()));
    }

    #[test]
    fn test_negative_decimal() {
        assert_eq!(
            parse("-3.14"),
            Some("rhin teen dashmlav ek chaar".to_string())
        );
    }

    #[test]
    fn test_larger_decimal() {
        assert_eq!(
            parse("100.25"),
            Some("ek sau dashmlav do paanch".to_string())
        );
    }

    #[test]
    fn test_with_quantity() {
        assert_eq!(
            parse("1.5 lakh"),
            Some("ek dashmlav paanch lakh".to_string())
        );
        assert_eq!(
            parse("4.85 crore"),
            Some("chaar dashmlav aath paanch crore".to_string())
        );
    }

    #[test]
    fn test_non_decimal() {
        assert_eq!(parse("123"), None);
        assert_eq!(parse("hello"), None);
        assert_eq!(parse(""), None);
    }
}
