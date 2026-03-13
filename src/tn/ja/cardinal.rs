//! Cardinal TN tagger for Japanese (romaji output).
//!
//! Converts written cardinal numbers to spoken Japanese in romaji:
//! - "123" → "hyaku ni juu san"
//! - "-42" → "mainasu yon juu ni"
//! - "10000" → "ichi man"

use super::number_to_words;

/// Parse a written cardinal number to spoken Japanese words in romaji.
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

    // Must be digits (with optional commas, dots, or spaces as thousands separators)
    if !digits_part
        .chars()
        .all(|c| c.is_ascii_digit() || c == ',' || c == '.' || c == ' ' || c == '\u{a0}')
    {
        return None;
    }

    if !digits_part.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }

    // Strip thousands separators
    let clean: String = digits_part.chars().filter(|c| c.is_ascii_digit()).collect();
    let n: i64 = clean.parse().ok()?;

    if is_negative {
        Some(format!("mainasu {}", number_to_words(n)))
    } else {
        Some(number_to_words(n))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(parse("0"), Some("zero".to_string()));
        assert_eq!(parse("1"), Some("ichi".to_string()));
        assert_eq!(parse("21"), Some("ni juu ichi".to_string()));
        assert_eq!(parse("100"), Some("hyaku".to_string()));
        assert_eq!(parse("123"), Some("hyaku ni juu san".to_string()));
    }

    #[test]
    fn test_thousands_separators() {
        assert_eq!(parse("1 000"), Some("sen".to_string()));
        assert_eq!(parse("1,000"), Some("sen".to_string()));
        assert_eq!(parse("1.000"), Some("sen".to_string()));
        assert_eq!(parse("1 000 000"), Some("hyaku man".to_string()));
    }

    #[test]
    fn test_negative() {
        assert_eq!(parse("-42"), Some("mainasu yon juu ni".to_string()));
        assert_eq!(parse("-300"), Some("mainasu sanbyaku".to_string()));
        assert_eq!(parse("-1"), Some("mainasu ichi".to_string()));
        assert_eq!(parse("-10000"), Some("mainasu ichi man".to_string()));
    }

    #[test]
    fn test_large_numbers() {
        assert_eq!(parse("10000"), Some("ichi man".to_string()));
        assert_eq!(parse("3000"), Some("sanzen".to_string()));
        assert_eq!(parse("8000"), Some("hassen".to_string()));
    }

    #[test]
    fn test_non_numbers() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("12abc"), None);
        assert_eq!(parse(""), None);
    }
}
