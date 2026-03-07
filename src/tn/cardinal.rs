//! Cardinal TN tagger.
//!
//! Converts written cardinal numbers to spoken form:
//! - "123" → "one hundred twenty three"
//! - "-42" → "minus forty two"
//! - "1,000" → "one thousand"
//! - "0" → "zero"

use super::number_to_words;

/// Parse a written cardinal number to spoken words.
///
/// Detects pure digit strings with optional leading minus and commas.
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

    // Must be digits (with optional commas)
    if !digits_part
        .chars()
        .all(|c| c.is_ascii_digit() || c == ',')
    {
        return None;
    }

    // Must contain at least one digit
    if !digits_part.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }

    // Strip commas and parse
    let clean: String = digits_part.chars().filter(|c| *c != ',').collect();
    let n: i64 = clean.parse().ok()?;

    if is_negative {
        Some(format!("minus {}", number_to_words(n)))
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
        assert_eq!(parse("1"), Some("one".to_string()));
        assert_eq!(parse("21"), Some("twenty one".to_string()));
        assert_eq!(parse("100"), Some("one hundred".to_string()));
        assert_eq!(
            parse("123"),
            Some("one hundred twenty three".to_string())
        );
    }

    #[test]
    fn test_commas() {
        assert_eq!(parse("1,000"), Some("one thousand".to_string()));
        assert_eq!(
            parse("1,000,000"),
            Some("one million".to_string())
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
