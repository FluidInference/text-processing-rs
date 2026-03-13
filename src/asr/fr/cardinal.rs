//! Cardinal number tagger for French.
//!
//! Converts spoken French number words to digits:
//! - "un" → "1"
//! - "vingt et un" → "21"
//! - "cent vingt-trois" → "123"
//! - "mille deux cent trente-quatre" → "1234"
//! - "moins soixante" → "-60"

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    /// Single digit and teen numbers
    static ref ONES: HashMap<&'static str, i64> = {
        let mut m = HashMap::new();
        m.insert("zero", 0);
        m.insert("un", 1);
        m.insert("une", 1);
        m.insert("deux", 2);
        m.insert("trois", 3);
        m.insert("quatre", 4);
        m.insert("cinq", 5);
        m.insert("six", 6);
        m.insert("sept", 7);
        m.insert("huit", 8);
        m.insert("neuf", 9);
        m.insert("dix", 10);
        m.insert("onze", 11);
        m.insert("douze", 12);
        m.insert("treize", 13);
        m.insert("quatorze", 14);
        m.insert("quinze", 15);
        m.insert("seize", 16);
        m
    };

    /// Tens (30, 40, 50, 60) - Note: vingt (20) is handled specially for quatre-vingts
    static ref TENS: HashMap<&'static str, i64> = {
        let mut m = HashMap::new();
        m.insert("trente", 30);
        m.insert("quarante", 40);
        m.insert("cinquante", 50);
        m.insert("soixante", 60);
        // Belgian/Swiss French
        m.insert("septante", 70);
        m.insert("huitante", 80);
        m.insert("octante", 80);
        m.insert("nonante", 90);
        m
    };

    /// Scale words
    static ref SCALES: HashMap<&'static str, i128> = {
        let mut m = HashMap::new();
        m.insert("cent", 100);
        m.insert("cents", 100);
        m.insert("mille", 1_000);
        m.insert("million", 1_000_000);
        m.insert("millions", 1_000_000);
        m.insert("milliard", 1_000_000_000);
        m.insert("milliards", 1_000_000_000);
        m.insert("billion", 1_000_000_000_000);
        m.insert("billions", 1_000_000_000_000);
        m.insert("billiard", 1_000_000_000_000_000);
        m.insert("billiards", 1_000_000_000_000_000);
        m.insert("trillion", 1_000_000_000_000_000_000);
        m.insert("trillions", 1_000_000_000_000_000_000);
        m
    };
}

/// Parse spoken French cardinal number to string representation.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trim = input_lower.trim();

    if input_trim == "zero" {
        return Some("zero".to_string());
    }

    // Don't parse single digit words (0-9)
    let single_digits = [
        "un", "une", "deux", "trois", "quatre",
        "cinq", "six", "sept", "huit", "neuf",
    ];
    if single_digits.contains(&input_trim) {
        return None;
    }

    // Don't parse space-separated simple compounds without scale words or "et"
    // E.g. "quarante trois" should not parse, but "vingt et un" and "cent vingt" should
    if input_trim.contains(' ') && !contains_scale_word(input_trim) && !input_trim.contains(" et ") {
        // Special case: "moins" + single word (like "moins soixante")
        if !input_trim.starts_with("moins ") || input_trim.matches(' ').count() > 1 {
            return None;
        }
    }

    // Check for negative
    let (is_negative, rest) = if input_trim.starts_with("moins ") {
        (true, input_trim.strip_prefix("moins ")?)
    } else {
        (false, input_trim)
    };

    let num = words_to_number(rest)?;

    if is_negative {
        Some(format!("-{}", num))
    } else {
        Some(num.to_string())
    }
}

/// Check if input contains scale words (cent, mille, million, etc.)
fn contains_scale_word(input: &str) -> bool {
    let scale_words = [
        "cent", "cents",
        "mille", "mil",
        "million", "millions",
        "milliard", "milliards",
        "billion", "billions",
        "billiard", "billiards",
        "trillion", "trillions",
    ];
    scale_words.iter().any(|&word| input.contains(word))
}

pub fn words_to_number(input: &str) -> Option<i128> {
    // Normalize: remove hyphens, "et" connectors
    let normalized = input
        .replace("-", " ")
        .replace(" et ", " ")
        .replace("  ", " ");

    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    if tokens.is_empty() {
        return None;
    }

    let mut result: i128 = 0;
    let mut current: i128 = 0;
    let mut last_val: i128 = 0; // Track last value added for "quatre-vingt" handling

    for token in tokens {
        // Check if it's a scale word
        if let Some(&scale) = SCALES.get(token) {
            if scale == 100 {
                // "cent" multiplies current or assumes 1
                if current == 0 {
                    current = 100;
                } else {
                    current *= 100;
                }
                last_val = 0;
            } else {
                // "mille", "million", etc.
                if current == 0 {
                    current = 1; // "mille" = 1000, not 0
                }
                result += current * scale;
                current = 0;
                last_val = 0;
            }
        } else if let Some(&val) = ONES.get(token) {
            current += val as i128;
            last_val = val as i128;
        } else if let Some(&val) = TENS.get(token) {
            current += val as i128;
            last_val = val as i128;
        } else if token == "dix" {
            // Special handling for "soixante-dix" (70), "quatre-vingt-dix" (90)
            current += 10;
            last_val = 10;
        } else if token == "vingts" || token == "vingt" {
            // "quatre-vingts" = 4 * 20, check LAST value added, not total current
            if last_val >= 2 && last_val <= 4 {
                // Remove the last value and multiply by 20
                current = current - last_val + (last_val * 20);
                last_val = last_val * 20;
            } else {
                current += 20;
                last_val = 20;
            }
        } else {
            return None; // Unknown word
        }
    }

    result += current;

    if result == 0 {
        None
    } else {
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(parse("zero"), Some("zero".to_string()));
        // Single-digit words ("un", "deux", etc.) intentionally return None
        // to avoid over-matching in sentence context
        assert_eq!(parse("un"), None);
        assert_eq!(parse("deux"), None);
        assert_eq!(parse("dix"), Some("10".to_string()));
        assert_eq!(parse("seize"), Some("16".to_string()));
    }

    #[test]
    fn test_tens() {
        assert_eq!(parse("vingt"), Some("20".to_string()));
        assert_eq!(parse("vingt et un"), Some("21".to_string()));
        assert_eq!(parse("vingt-deux"), Some("22".to_string()));
        assert_eq!(parse("trente"), Some("30".to_string()));
    }

    #[test]
    fn test_special() {
        assert_eq!(parse("soixante-dix"), Some("70".to_string()));
        assert_eq!(parse("quatre-vingts"), Some("80".to_string()));
        assert_eq!(parse("quatre-vingt-dix"), Some("90".to_string()));
        assert_eq!(parse("quatre-vingt-dix-neuf"), Some("99".to_string()));
    }

    #[test]
    fn test_hundreds() {
        assert_eq!(parse("cent"), Some("100".to_string()));
        assert_eq!(parse("deux cents"), Some("200".to_string()));
        assert_eq!(parse("deux cent vingt"), Some("220".to_string()));
    }

    #[test]
    fn test_thousands() {
        assert_eq!(parse("mille"), Some("1000".to_string()));
        assert_eq!(parse("deux mille"), Some("2000".to_string()));
        assert_eq!(parse("deux mille vingt-cinq"), Some("2025".to_string()));
    }

    #[test]
    fn test_large() {
        assert_eq!(parse("un million"), Some("1000000".to_string()));
        assert_eq!(parse("deux millions trois"), Some("2000003".to_string()));
    }

    #[test]
    fn test_negative() {
        assert_eq!(parse("moins quarante-deux"), Some("-42".to_string()));
        assert_eq!(parse("moins mille"), Some("-1000".to_string()));
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse(""), None);
    }
}
