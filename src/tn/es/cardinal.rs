//! Cardinal TN tagger for Spanish.
//!
//! Converts written cardinal numbers to spoken Spanish:
//! - "123" → "ciento veintitres"
//! - "-42" → "menos cuarenta y dos"
//! - "1 000" → "mil"

use super::number_to_words;

/// Parse a written cardinal number to spoken Spanish words.
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

    // Must be digits (with optional dots, commas, or spaces as thousands separators)
    if !digits_part
        .chars()
        .all(|c| c.is_ascii_digit() || c == ',' || c == '.' || c == ' ' || c == '\u{a0}')
    {
        return None;
    }

    if !digits_part.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }

    // Strip thousands separators (spaces, dots used as thousands sep in Spanish)
    let clean: String = digits_part.chars().filter(|c| c.is_ascii_digit()).collect();
    let n: i64 = clean.parse().ok()?;

    if is_negative {
        Some(format!("menos {}", number_to_words(n)))
    } else {
        Some(number_to_words(n))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(parse("0"), Some("cero".to_string()));
        assert_eq!(parse("1"), Some("uno".to_string()));
        assert_eq!(parse("21"), Some("veintiuno".to_string()));
        assert_eq!(parse("100"), Some("cien".to_string()));
        assert_eq!(parse("123"), Some("ciento veintitres".to_string()));
    }

    #[test]
    fn test_thousands_separators() {
        assert_eq!(parse("1 000"), Some("mil".to_string()));
        assert_eq!(parse("1.000"), Some("mil".to_string()));
        assert_eq!(parse("1,000"), Some("mil".to_string()));
        assert_eq!(parse("1 000 000"), Some("un millon".to_string()));
    }

    #[test]
    fn test_negative() {
        assert_eq!(parse("-42"), Some("menos cuarenta y dos".to_string()));
        assert_eq!(parse("-1"), Some("menos uno".to_string()));
        assert_eq!(parse("-1000"), Some("menos mil".to_string()));
    }

    #[test]
    fn test_non_numbers() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("12abc"), None);
        assert_eq!(parse(""), None);
    }
}
