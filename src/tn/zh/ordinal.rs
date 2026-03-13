//! Ordinal TN tagger for Mandarin Chinese.
//!
//! Converts written ordinal numbers to spoken Mandarin pinyin:
//! - "第1" -> "di yi"
//! - "第2" -> "di er"
//! - "第100" -> "di yi bai"
//!
//! Chinese ordinals are formed by prefixing "di" (第) to the cardinal number.

use super::number_to_words;

/// Parse a written ordinal to spoken Mandarin pinyin words.
///
/// Supports formats:
/// - Chinese style: "第1", "第2", "第100"
/// - English suffix style: "1st", "2nd", "3rd", "4th" (also converted to Chinese ordinals)
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // Try Chinese ordinal prefix: 第N
    if let Some(num_str) = trimmed.strip_prefix('\u{7B2C}') {
        // 第 = U+7B2C
        let num_str = num_str.trim();
        if num_str.is_empty() || !num_str.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        let n: i64 = num_str.parse().ok()?;
        if n <= 0 {
            return None;
        }
        return Some(format!("di {}", number_to_words(n)));
    }

    // Try English ordinal suffixes: 1st, 2nd, 3rd, 4th, etc.
    let num_str = if let Some(s) = trimmed.strip_suffix("st") {
        s
    } else if let Some(s) = trimmed.strip_suffix("nd") {
        s
    } else if let Some(s) = trimmed.strip_suffix("rd") {
        s
    } else if let Some(s) = trimmed.strip_suffix("th") {
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

    Some(format!("di {}", number_to_words(n)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chinese_ordinal() {
        assert_eq!(parse("\u{7B2C}1"), Some("di yi".to_string()));
        assert_eq!(parse("\u{7B2C}2"), Some("di er".to_string()));
        assert_eq!(parse("\u{7B2C}10"), Some("di shi".to_string()));
        assert_eq!(parse("\u{7B2C}100"), Some("di yi bai".to_string()));
    }

    #[test]
    fn test_english_suffix() {
        assert_eq!(parse("1st"), Some("di yi".to_string()));
        assert_eq!(parse("2nd"), Some("di er".to_string()));
        assert_eq!(parse("3rd"), Some("di san".to_string()));
        assert_eq!(parse("4th"), Some("di si".to_string()));
        assert_eq!(parse("21st"), Some("di er shi yi".to_string()));
    }

    #[test]
    fn test_non_ordinals() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("0th"), None);
        assert_eq!(parse("\u{7B2C}0"), None);
        assert_eq!(parse(""), None);
    }
}
