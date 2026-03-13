//! Cardinal TN tagger for French.
//!
//! Converts written cardinal numbers to spoken French:
//! - "123" → "cent vingt-trois"
//! - "-42" → "moins quarante-deux"
//! - "1 000" → "mille"

use super::number_to_words;

/// Parse a written cardinal number to spoken French words.
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

    // Strip thousands separators (spaces, dots, commas used as thousands sep)
    // French uses space or dot as thousands separator
    let clean: String = digits_part.chars().filter(|c| c.is_ascii_digit()).collect();
    let n: i64 = clean.parse().ok()?;

    if is_negative {
        Some(format!("moins {}", number_to_words(n)))
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
        assert_eq!(parse("1"), Some("un".to_string()));
        assert_eq!(parse("21"), Some("vingt et un".to_string()));
        assert_eq!(parse("100"), Some("cent".to_string()));
        assert_eq!(parse("123"), Some("cent vingt-trois".to_string()));
    }

    #[test]
    fn test_thousands_separators() {
        assert_eq!(parse("1 000"), Some("mille".to_string()));
        assert_eq!(parse("1.000"), Some("mille".to_string()));
        assert_eq!(parse("1,000"), Some("mille".to_string()));
        assert_eq!(parse("1 000 000"), Some("un million".to_string()));
    }

    #[test]
    fn test_negative() {
        assert_eq!(parse("-42"), Some("moins quarante-deux".to_string()));
        assert_eq!(parse("-1000"), Some("moins mille".to_string()));
    }

    #[test]
    fn test_non_numbers() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("12abc"), None);
        assert_eq!(parse(""), None);
    }
}
