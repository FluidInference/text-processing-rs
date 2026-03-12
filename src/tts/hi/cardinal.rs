//! Cardinal TN tagger for Hindi (romanized).
//!
//! Converts written cardinal numbers to spoken romanized Hindi:
//! - "123" → "ek sau teis"
//! - "-42" → "rhin bayaalees"
//! - "1,00,000" → "ek lakh"

use super::number_to_words;

/// Parse a written cardinal number to spoken romanized Hindi words.
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
    // Hindi uses Indian comma grouping: 1,23,45,678
    if !digits_part
        .chars()
        .all(|c| c.is_ascii_digit() || c == ',' || c == '.' || c == ' ' || c == '\u{a0}')
    {
        return None;
    }

    if !digits_part.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }

    // Strip thousands/lakh separators
    let clean: String = digits_part
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect();
    let n: i64 = clean.parse().ok()?;

    if is_negative {
        Some(format!("rhin {}", number_to_words(n)))
    } else {
        Some(number_to_words(n))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(parse("0"), Some("shunya".to_string()));
        assert_eq!(parse("1"), Some("ek".to_string()));
        assert_eq!(parse("21"), Some("ikkees".to_string()));
        assert_eq!(parse("100"), Some("ek sau".to_string()));
        assert_eq!(parse("123"), Some("ek sau teis".to_string()));
    }

    #[test]
    fn test_thousands_separators() {
        assert_eq!(parse("1 000"), Some("ek hazaar".to_string()));
        assert_eq!(parse("1,000"), Some("ek hazaar".to_string()));
        assert_eq!(parse("1.000"), Some("ek hazaar".to_string()));
        assert_eq!(parse("1 000 000"), Some("das lakh".to_string()));
    }

    #[test]
    fn test_negative() {
        assert_eq!(parse("-42"), Some("rhin bayaalees".to_string()));
        assert_eq!(parse("-1"), Some("rhin ek".to_string()));
        assert_eq!(parse("-1000"), Some("rhin ek hazaar".to_string()));
    }

    #[test]
    fn test_indian_grouping() {
        assert_eq!(parse("1,00,000"), Some("ek lakh".to_string()));
        assert_eq!(parse("1,00,00,000"), Some("ek crore".to_string()));
    }

    #[test]
    fn test_non_numbers() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("12abc"), None);
        assert_eq!(parse(""), None);
    }
}
