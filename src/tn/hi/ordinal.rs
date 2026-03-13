//! Ordinal TN tagger for Hindi (romanized).
//!
//! Converts written ordinal numbers to spoken romanized Hindi:
//! - "1st" → "pehla"
//! - "2nd" → "doosra"
//! - "3rd" → "teesra"
//! - "5th" → "paanchvaan"

use super::number_to_words;

/// Special ordinal forms for small numbers in Hindi.
const SPECIAL_ORDINALS: &[(i64, &str)] = &[
    (1, "pehla"),
    (2, "doosra"),
    (3, "teesra"),
    (4, "chautha"),
    (5, "paanchvaan"),
    (6, "chhathvaan"),
    (7, "saatvaan"),
    (8, "aathvaan"),
    (9, "nauvaan"),
    (10, "dasvaan"),
];

/// Parse a written ordinal to spoken romanized Hindi words.
pub fn parse(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // Detect English ordinal suffixes: st, nd, rd, th
    // Also detect Hindi-style suffix: vaan, veen, va, vi
    let num_str = if let Some(s) = trimmed.strip_suffix("vaan") {
        s
    } else if let Some(s) = trimmed.strip_suffix("veen") {
        s
    } else if let Some(s) = trimmed.strip_suffix("th") {
        s
    } else if let Some(s) = trimmed.strip_suffix("st") {
        s
    } else if let Some(s) = trimmed.strip_suffix("nd") {
        s
    } else if let Some(s) = trimmed.strip_suffix("rd") {
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

    // Check for special ordinal forms
    for &(val, word) in SPECIAL_ORDINALS {
        if n == val {
            return Some(word.to_string());
        }
    }

    // General ordinal: cardinal + "vaan"
    let cardinal = number_to_words(n);
    Some(format!("{}vaan", cardinal))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_special_ordinals() {
        assert_eq!(parse("1st"), Some("pehla".to_string()));
        assert_eq!(parse("2nd"), Some("doosra".to_string()));
        assert_eq!(parse("3rd"), Some("teesra".to_string()));
        assert_eq!(parse("4th"), Some("chautha".to_string()));
    }

    #[test]
    fn test_general_ordinals() {
        assert_eq!(parse("5th"), Some("paanchvaan".to_string()));
        assert_eq!(parse("20th"), Some("beesvaan".to_string()));
        assert_eq!(parse("100th"), Some("ek sauvaan".to_string()));
    }

    #[test]
    fn test_hindi_suffix() {
        assert_eq!(parse("5vaan"), Some("paanchvaan".to_string()));
        assert_eq!(parse("10vaan"), Some("dasvaan".to_string()));
    }

    #[test]
    fn test_non_ordinals() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("0th"), None);
        assert_eq!(parse(""), None);
    }
}
