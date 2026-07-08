//! Decimal TN tagger.
//!
//! Converts written decimal numbers to spoken form:
//! - "3.14" → "three point one four"
//! - "0.5" → "zero point five"
//! - ".1665" → "point one six six five" (no integer part → no "zero")
//! - "1.5 billion" → "one point five billion"
//! - "100 million" → "one hundred million" (integer + scale word)
//! - "0.1 M" → "zero point one M" (magnitude abbreviation kept literally)

use super::{number_to_words, spell_digits};

/// Spelled-out scale words recognized after a number.
const QUANTITY_SUFFIXES: &[&str] = &[
    "billion",
    "million",
    "trillion",
    "quadrillion",
    "quintillion",
    "thousand",
];

/// Single-letter magnitude abbreviations kept verbatim after a number
/// (`0.1 B` → "zero point one B"), matching NeMo. Uppercase only — a
/// lowercase `m`/`b` is a measurement unit handled by the measure tagger.
const MAGNITUDE_ABBREVS: &[&str] = &["K", "M", "B", "G", "T"];

/// Parse a written decimal (or integer-with-scale) number to spoken form.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let (number_part, suffix) = extract_suffix(trimmed);

    let core = if number_part.contains('.') {
        spell_decimal(number_part)?
    } else {
        // A bare integer only belongs to this tagger when it carries a scale
        // suffix ("100 million"); otherwise it is the cardinal tagger's job.
        suffix?;
        spell_integer(number_part)?
    };

    Some(match suffix {
        Some(suf) => format!("{} {}", core, suf),
        None => core,
    })
}

/// Spell `[-]INT.FRAC`; an empty integer part omits the leading "zero"
/// (`.1665` → "point one six six five").
fn spell_decimal(number_part: &str) -> Option<String> {
    let (int_str, frac_str) = number_part.split_once('.')?;

    let (is_negative, int_digits) = match int_str.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, int_str),
    };

    if !int_digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    if frac_str.is_empty() || !frac_str.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    let frac_words = spell_digits(frac_str);
    let sign = if is_negative { "minus " } else { "" };

    if int_digits.is_empty() {
        Some(format!("{}point {}", sign, frac_words))
    } else {
        let int_val: i64 = int_digits.parse().ok()?;
        Some(format!(
            "{}{} point {}",
            sign,
            number_to_words(int_val),
            frac_words
        ))
    }
}

/// Spell a bare `[-]INT` (used only alongside a scale suffix).
fn spell_integer(number_part: &str) -> Option<String> {
    let (is_negative, digits) = match number_part.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, number_part),
    };
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let val: i64 = digits.parse().ok()?;
    let sign = if is_negative { "minus " } else { "" };
    Some(format!("{}{}", sign, number_to_words(val)))
}

/// Split off a trailing scale word or magnitude abbreviation.
fn extract_suffix(input: &str) -> (&str, Option<&str>) {
    for &suf in QUANTITY_SUFFIXES {
        if let Some(before) = input.strip_suffix(suf) {
            let before = before.trim_end();
            if !before.is_empty() {
                return (before, Some(suf));
            }
        }
    }
    // Magnitude abbreviations require a space so we never split a bare token.
    for &suf in MAGNITUDE_ABBREVS {
        if let Some(before) = input.strip_suffix(suf) {
            if before.ends_with(' ') {
                let before = before.trim_end();
                if !before.is_empty() {
                    return (before, Some(suf));
                }
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
    fn test_leading_dot_omits_zero() {
        assert_eq!(parse(".1665"), Some("point one six six five".to_string()));
        assert_eq!(parse(".1 trillion"), Some("point one trillion".to_string()));
    }

    #[test]
    fn test_integer_with_scale_word() {
        assert_eq!(
            parse("100 million"),
            Some("one hundred million".to_string())
        );
        // A bare integer without a scale word is not this tagger's job.
        assert_eq!(parse("100"), None);
    }

    #[test]
    fn test_magnitude_abbreviation() {
        assert_eq!(parse("0.1 M"), Some("zero point one M".to_string()));
        assert_eq!(parse("0.1 B"), Some("zero point one B".to_string()));
        assert_eq!(parse("0.1 K"), Some("zero point one K".to_string()));
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
