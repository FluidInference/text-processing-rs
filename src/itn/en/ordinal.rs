//! Ordinal number tagger.
//!
//! Converts spoken ordinal numbers to written form:
//! - "first" → "1st"
//! - "twenty first" → "21st"
//! - "one hundredth" → "100th"

use lazy_static::lazy_static;
use std::collections::HashMap;

use super::cardinal::words_to_number;

lazy_static! {
    /// Ordinal words mapping to (suffix, value)
    static ref ORDINAL_ONES: HashMap<&'static str, i64> = {
        let mut m = HashMap::new();
        m.insert("zeroth", 0);
        m.insert("first", 1);
        m.insert("second", 2);
        m.insert("third", 3);
        m.insert("fourth", 4);
        m.insert("fifth", 5);
        m.insert("sixth", 6);
        m.insert("seventh", 7);
        m.insert("eighth", 8);
        m.insert("ninth", 9);
        m.insert("tenth", 10);
        m.insert("eleventh", 11);
        m.insert("twelfth", 12);
        m.insert("thirteenth", 13);
        m.insert("fourteenth", 14);
        m.insert("fifteenth", 15);
        m.insert("sixteenth", 16);
        m.insert("seventeenth", 17);
        m.insert("eighteenth", 18);
        m.insert("nineteenth", 19);
        m
    };

    /// Ordinal tens
    static ref ORDINAL_TENS: HashMap<&'static str, i64> = {
        let mut m = HashMap::new();
        m.insert("twentieth", 20);
        m.insert("thirtieth", 30);
        m.insert("fortieth", 40);
        m.insert("fiftieth", 50);
        m.insert("sixtieth", 60);
        m.insert("seventieth", 70);
        m.insert("eightieth", 80);
        m.insert("ninetieth", 90);
        m
    };

    /// Ordinal scales
    static ref ORDINAL_SCALES: HashMap<&'static str, i64> = {
        let mut m = HashMap::new();
        m.insert("hundredth", 100);
        m.insert("thousandth", 1000);
        m.insert("millionth", 1_000_000);
        m.insert("billionth", 1_000_000_000);
        m
    };
}

/// Parse spoken ordinal to written form.
pub fn parse(input: &str) -> Option<String> {
    let input = input.to_lowercase();
    let words: Vec<&str> = input.split_whitespace().collect();

    if words.is_empty() {
        return None;
    }

    let last_word = *words.last()?;

    // Check if last word is an ordinal
    let ordinal_value = get_ordinal_value(last_word)?;

    if words.len() == 1 {
        // Single ordinal word
        return Some(format_ordinal(ordinal_value));
    }

    // Multiple words: parse cardinal prefix + ordinal suffix
    // e.g., "twenty first" = 20 + 1 = 21st
    // e.g., "one hundred twenty first" = 100 + 20 + 1 = 121st

    let prefix_words = &words[..words.len() - 1];
    let prefix = prefix_words.join(" ");

    // Parse the cardinal prefix
    let prefix_value = words_to_number(&prefix)? as i64;

    // Special case: ordinal scales like "hundredth", "thousandth"
    if let Some(&scale) = ORDINAL_SCALES.get(last_word) {
        // "one hundredth" = 1 * 100 = 100th
        // "twenty five thousandth" = 25 * 1000 = 25000th
        return Some(format_ordinal(prefix_value * scale));
    }

    // Regular ordinal: add prefix + ordinal value
    Some(format_ordinal(prefix_value + ordinal_value))
}

/// Get the numeric value of an ordinal word.
fn get_ordinal_value(word: &str) -> Option<i64> {
    if let Some(&val) = ORDINAL_ONES.get(word) {
        return Some(val);
    }
    if let Some(&val) = ORDINAL_TENS.get(word) {
        return Some(val);
    }
    if let Some(&val) = ORDINAL_SCALES.get(word) {
        return Some(val);
    }
    None
}

/// Format a number as an ordinal (1st, 2nd, 3rd, 4th, etc.)
fn format_ordinal(n: i64) -> String {
    let suffix = match n % 100 {
        11 | 12 | 13 => "th",
        _ => match n % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        },
    };
    format!("{}{}", n, suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ones() {
        assert_eq!(parse("first"), Some("1st".to_string()));
        assert_eq!(parse("second"), Some("2nd".to_string()));
        assert_eq!(parse("third"), Some("3rd".to_string()));
        assert_eq!(parse("fourth"), Some("4th".to_string()));
        assert_eq!(parse("fifth"), Some("5th".to_string()));
    }

    #[test]
    fn test_teens() {
        assert_eq!(parse("eleventh"), Some("11th".to_string()));
        assert_eq!(parse("twelfth"), Some("12th".to_string()));
        assert_eq!(parse("thirteenth"), Some("13th".to_string()));
    }

    #[test]
    fn test_tens() {
        assert_eq!(parse("twentieth"), Some("20th".to_string()));
        assert_eq!(parse("twenty first"), Some("21st".to_string()));
        assert_eq!(parse("twenty second"), Some("22nd".to_string()));
        assert_eq!(parse("twenty third"), Some("23rd".to_string()));
        assert_eq!(parse("forty second"), Some("42nd".to_string()));
    }

    #[test]
    fn test_hundreds() {
        assert_eq!(parse("one hundredth"), Some("100th".to_string()));
        assert_eq!(parse("one hundred first"), Some("101st".to_string()));
        assert_eq!(parse("one hundred eleventh"), Some("111th".to_string()));
        assert_eq!(parse("one hundred twenty first"), Some("121st".to_string()));
    }

    #[test]
    fn test_thousands() {
        assert_eq!(parse("one thousandth"), Some("1000th".to_string()));
        assert_eq!(
            parse("eleven hundred twenty first"),
            Some("1121st".to_string())
        );
    }

    #[test]
    fn test_zeroth() {
        assert_eq!(parse("zeroth"), Some("0th".to_string()));
    }
}
