//! Cardinal TN tagger for Mandarin Chinese.
//!
//! Converts written cardinal numbers to spoken Mandarin pinyin:
//! - "123" -> "yi bai er shi san"
//! - "-42" -> "fu si shi er"
//! - "10000" -> "yi wan"

use super::number_to_words;

/// Parse a written cardinal number to spoken Mandarin pinyin words.
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
        Some(format!("fu {}", number_to_words(n)))
    } else {
        Some(number_to_words(n))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(parse("0"), Some("ling".to_string()));
        assert_eq!(parse("1"), Some("yi".to_string()));
        assert_eq!(parse("21"), Some("er shi yi".to_string()));
        assert_eq!(parse("100"), Some("yi bai".to_string()));
        assert_eq!(parse("123"), Some("yi bai er shi san".to_string()));
    }

    #[test]
    fn test_wan_grouping() {
        assert_eq!(parse("10000"), Some("yi wan".to_string()));
        assert_eq!(
            parse("12345"),
            Some("yi wan er qian san bai si shi wu".to_string())
        );
        assert_eq!(parse("100000000"), Some("yi yi".to_string()));
    }

    #[test]
    fn test_thousands_separators() {
        assert_eq!(parse("1 000"), Some("yi qian".to_string()));
        assert_eq!(parse("1,000"), Some("yi qian".to_string()));
        assert_eq!(parse("1.000"), Some("yi qian".to_string()));
        assert_eq!(parse("1 000 000"), Some("yi bai wan".to_string()));
    }

    #[test]
    fn test_negative() {
        assert_eq!(parse("-42"), Some("fu si shi er".to_string()));
        assert_eq!(parse("-1"), Some("fu yi".to_string()));
        assert_eq!(parse("-10000"), Some("fu yi wan".to_string()));
    }

    #[test]
    fn test_non_numbers() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("12abc"), None);
        assert_eq!(parse(""), None);
    }
}
