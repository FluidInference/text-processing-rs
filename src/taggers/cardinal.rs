//! Cardinal number tagger.
//!
//! Converts spoken number words to digits:
//! - "one" → "1"
//! - "twenty one" → "21"
//! - "one hundred twenty three" → "123"
//! - "one thousand two hundred thirty four" → "1234"
//! - "minus sixty" → "-60"

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    /// Single digit and teen numbers
    static ref ONES: HashMap<&'static str, i64> = {
        let mut m = HashMap::new();
        m.insert("zero", 0);
        m.insert("one", 1);
        m.insert("two", 2);
        m.insert("three", 3);
        m.insert("four", 4);
        m.insert("five", 5);
        m.insert("six", 6);
        m.insert("seven", 7);
        m.insert("eight", 8);
        m.insert("nine", 9);
        m.insert("ten", 10);
        m.insert("eleven", 11);
        m.insert("twelve", 12);
        m.insert("thirteen", 13);
        m.insert("fourteen", 14);
        m.insert("fifteen", 15);
        m.insert("sixteen", 16);
        m.insert("seventeen", 17);
        m.insert("eighteen", 18);
        m.insert("nineteen", 19);
        m
    };

    /// Tens (20, 30, 40, ...)
    static ref TENS: HashMap<&'static str, i64> = {
        let mut m = HashMap::new();
        m.insert("twenty", 20);
        m.insert("thirty", 30);
        m.insert("forty", 40);
        m.insert("fifty", 50);
        m.insert("sixty", 60);
        m.insert("seventy", 70);
        m.insert("eighty", 80);
        m.insert("ninety", 90);
        m
    };

    /// Scale words (using i128 to support sextillion and larger)
    static ref SCALES: HashMap<&'static str, i128> = {
        let mut m = HashMap::new();
        m.insert("hundred", 100);
        m.insert("thousand", 1_000);
        m.insert("million", 1_000_000);
        m.insert("billion", 1_000_000_000);
        m.insert("trillion", 1_000_000_000_000);
        m.insert("quadrillion", 1_000_000_000_000_000);
        m.insert("quintillion", 1_000_000_000_000_000_000);
        m.insert("sextillion", 1_000_000_000_000_000_000_000_i128);
        // Indian numbering system
        m.insert("lakh", 100_000);
        m.insert("crore", 10_000_000);
        m
    };
}

/// Parse spoken cardinal number to string representation.
///
/// Returns None if the input cannot be parsed as a number.
pub fn parse(input: &str) -> Option<String> {
    let input = input.to_lowercase();
    let input = input.trim();

    // Handle "zero" specially - NeMo returns "zero" not "0"
    if input == "zero" {
        return Some("zero".to_string());
    }

    // Check for negative
    let (is_negative, rest) = if input.starts_with("minus ") {
        (true, input.strip_prefix("minus ")?)
    } else if input.starts_with("negative ") {
        (true, input.strip_prefix("negative ")?)
    } else {
        (false, input)
    };

    let num = words_to_number(rest)?;

    if is_negative {
        Some(format!("-{}", num))
    } else {
        Some(num.to_string())
    }
}

/// Convert spoken number words to integer.
///
/// Algorithm:
/// 1. Tokenize input
/// 2. Process left-to-right, accumulating values
/// 3. Scale words (hundred, thousand, million) multiply the current accumulator
/// 4. Handle "and" as a separator (ignored)
///
/// Examples:
/// - "twenty one" → 20 + 1 = 21
/// - "one hundred twenty three" → (1 * 100) + 20 + 3 = 123
/// - "one thousand two hundred thirty four" → (1 * 1000) + (2 * 100) + 30 + 4 = 1234
pub fn words_to_number(input: &str) -> Option<i128> {
    let input = input.to_lowercase();
    let words: Vec<&str> = input
        .split_whitespace()
        .filter(|w| *w != "and" && *w != "a")
        .collect();

    if words.is_empty() {
        return None;
    }

    // Handle special case: "eleven hundred" = 1100
    if words.len() == 2 && words[1] == "hundred" {
        if let Some(&val) = ONES.get(words[0]) {
            if val >= 11 && val <= 19 {
                return Some((val * 100) as i128);
            }
        }
        if let Some(&val) = TENS.get(words[0]) {
            return Some((val * 100) as i128);
        }
    }

    // Handle "eleven hundred twenty one" pattern
    if words.len() >= 2 && words[1] == "hundred" {
        if let Some(&first_val) = ONES.get(words[0]) {
            if first_val >= 11 && first_val <= 99 {
                let base = (first_val * 100) as i128;
                if words.len() == 2 {
                    return Some(base);
                }
                // Parse remaining words
                let rest = words[2..].join(" ");
                if let Some(remainder) = words_to_number(&rest) {
                    return Some(base + remainder);
                }
            }
        }
        if let Some(&first_val) = TENS.get(words[0]) {
            let base = (first_val * 100) as i128;
            if words.len() == 2 {
                return Some(base);
            }
            let rest = words[2..].join(" ");
            if let Some(remainder) = words_to_number(&rest) {
                return Some(base + remainder);
            }
        }
    }

    let mut result: i128 = 0;
    let mut current: i128 = 0;
    let mut found_number = false;

    for word in words {
        if let Some(&val) = ONES.get(word) {
            current += val as i128;
            found_number = true;
        } else if let Some(&val) = TENS.get(word) {
            current += val as i128;
            found_number = true;
        } else if word == "hundred" {
            if current == 0 {
                current = 1;
            }
            current *= 100;
            found_number = true;
        } else if let Some(&scale) = SCALES.get(word) {
            if scale >= 1000 {
                if current == 0 {
                    current = 1;
                }
                current *= scale;
                result += current;
                current = 0;
                found_number = true;
            }
        } else {
            // Unknown word - not a valid number
            return None;
        }
    }

    if found_number {
        Some(result + current)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ones() {
        assert_eq!(parse("one"), Some("1".to_string()));
        assert_eq!(parse("two"), Some("2".to_string()));
        assert_eq!(parse("nine"), Some("9".to_string()));
        assert_eq!(parse("ten"), Some("10".to_string()));
        assert_eq!(parse("fifteen"), Some("15".to_string()));
        assert_eq!(parse("nineteen"), Some("19".to_string()));
    }

    #[test]
    fn test_tens() {
        assert_eq!(parse("twenty"), Some("20".to_string()));
        assert_eq!(parse("twenty one"), Some("21".to_string()));
        assert_eq!(parse("forty two"), Some("42".to_string()));
        assert_eq!(parse("ninety nine"), Some("99".to_string()));
    }

    #[test]
    fn test_hundreds() {
        assert_eq!(parse("one hundred"), Some("100".to_string()));
        assert_eq!(parse("one hundred one"), Some("101".to_string()));
        assert_eq!(parse("one hundred and one"), Some("101".to_string()));
        assert_eq!(parse("two hundred twenty two"), Some("222".to_string()));
    }

    #[test]
    fn test_eleven_hundred() {
        assert_eq!(parse("eleven hundred"), Some("1100".to_string()));
        assert_eq!(parse("twenty one hundred"), Some("2100".to_string()));
        assert_eq!(parse("eleven hundred twenty one"), Some("1121".to_string()));
    }

    #[test]
    fn test_thousands() {
        assert_eq!(parse("one thousand"), Some("1000".to_string()));
        assert_eq!(parse("one thousand one"), Some("1001".to_string()));
        assert_eq!(parse("one thousand one hundred"), Some("1100".to_string()));
        assert_eq!(
            parse("one thousand two hundred thirty four"),
            Some("1234".to_string())
        );
    }

    #[test]
    fn test_millions() {
        assert_eq!(parse("one million"), Some("1000000".to_string()));
        assert_eq!(parse("two million three"), Some("2000003".to_string()));
    }

    #[test]
    fn test_negative() {
        assert_eq!(parse("minus sixty"), Some("-60".to_string()));
        assert_eq!(
            parse("minus twenty five thousand thirty seven"),
            Some("-25037".to_string())
        );
    }

    #[test]
    fn test_zero() {
        assert_eq!(parse("zero"), Some("zero".to_string()));
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("one hello"), None);
    }
}
