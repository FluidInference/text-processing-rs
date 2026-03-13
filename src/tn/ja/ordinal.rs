//! Ordinal TN tagger for Japanese (romaji output).
//!
//! Converts written ordinal numbers to spoken Japanese in romaji:
//! - "第1" → "dai ichi"
//! - "第100" → "dai hyaku"
//! - "1st" → "dai ichi"
//! - "3rd" → "dai san"

use super::number_to_words;

/// Parse a written ordinal to spoken Japanese words in romaji.
///
/// Supports two formats:
/// - Japanese: "第1", "第100"
/// - English suffixes: "1st", "2nd", "3rd", "4th", etc.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // Try Japanese format: 第N
    if let Some(result) = parse_dai_format(trimmed) {
        return Some(result);
    }

    // Try English ordinal suffixes: 1st, 2nd, 3rd, 4th...
    if let Some(result) = parse_english_suffix(trimmed) {
        return Some(result);
    }

    None
}

fn parse_dai_format(input: &str) -> Option<String> {
    let rest = input.strip_prefix('\u{7B2C}')?; // 第
    if rest.is_empty() || !rest.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    let n: i64 = rest.parse().ok()?;
    if n <= 0 {
        return None;
    }

    Some(format!("dai {}", number_to_words(n)))
}

fn parse_english_suffix(input: &str) -> Option<String> {
    let num_str = if let Some(s) = input.strip_suffix("st") {
        s
    } else if let Some(s) = input.strip_suffix("nd") {
        s
    } else if let Some(s) = input.strip_suffix("rd") {
        s
    } else if let Some(s) = input.strip_suffix("th") {
        s
    } else {
        return None;
    };

    if num_str.is_empty() || !num_str.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    let n: i64 = num_str.parse().ok()?;
    if n <= 0 {
        return None;
    }

    Some(format!("dai {}", number_to_words(n)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dai_format() {
        assert_eq!(parse("\u{7B2C}1"), Some("dai ichi".to_string()));
        assert_eq!(parse("\u{7B2C}3"), Some("dai san".to_string()));
        assert_eq!(parse("\u{7B2C}100"), Some("dai hyaku".to_string()));
        assert_eq!(parse("\u{7B2C}10"), Some("dai juu".to_string()));
    }

    #[test]
    fn test_english_suffix() {
        assert_eq!(parse("1st"), Some("dai ichi".to_string()));
        assert_eq!(parse("2nd"), Some("dai ni".to_string()));
        assert_eq!(parse("3rd"), Some("dai san".to_string()));
        assert_eq!(parse("4th"), Some("dai yon".to_string()));
        assert_eq!(parse("10th"), Some("dai juu".to_string()));
        assert_eq!(parse("21st"), Some("dai ni juu ichi".to_string()));
    }

    #[test]
    fn test_non_ordinals() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("0th"), None);
        assert_eq!(parse(""), None);
    }
}
