//! Cardinal TN tagger for German.
//!
//! Converts written cardinal numbers to spoken German:
//! - "123" → "einhundertdreiundzwanzig"
//! - "-42" → "minus zweiundvierzig"
//! - "1.000" → "eintausend" (dot as thousands separator)

use super::number_to_words;

/// Parse a written cardinal number to spoken German words.
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

    // Must be digits (with optional dots, spaces, or non-breaking spaces as thousands separators)
    // German uses dot or space as thousands separator (e.g. "1.000" or "1 000")
    if !digits_part
        .chars()
        .all(|c| c.is_ascii_digit() || c == '.' || c == ' ' || c == '\u{a0}')
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
        assert_eq!(parse("0"), Some("null".to_string()));
        assert_eq!(parse("1"), Some("eins".to_string()));
        assert_eq!(parse("21"), Some("einundzwanzig".to_string()));
        assert_eq!(parse("100"), Some("einhundert".to_string()));
        assert_eq!(parse("123"), Some("einhundertdreiundzwanzig".to_string()));
    }

    #[test]
    fn test_negative() {
        assert_eq!(parse("-42"), Some("minus zweiundvierzig".to_string()));
        assert_eq!(parse("-1"), Some("minus eins".to_string()));
        assert_eq!(parse("-1000"), Some("minus eintausend".to_string()));
    }

    #[test]
    fn test_thousands_separator() {
        assert_eq!(parse("1.000"), Some("eintausend".to_string()));
        assert_eq!(
            parse("2.025"),
            Some("zweitausendfuenfundzwanzig".to_string())
        );
    }

    #[test]
    fn test_non_numbers() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("12abc"), None);
        assert_eq!(parse(""), None);
    }
}
