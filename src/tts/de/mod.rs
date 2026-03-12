//! Text Normalization taggers for German.
//!
//! Converts written-form text to spoken German:
//! - "200" → "zweihundert"
//! - "5,50 €" → "fuenf euro und fuenfzig cent"
//! - "5. Januar 2025" → "fuenfter januar zweitausendfuenfundzwanzig"

pub mod cardinal;
pub mod date;
pub mod decimal;
pub mod electronic;
pub mod measure;
pub mod money;
pub mod ordinal;
pub mod telephone;
pub mod time;
pub mod whitelist;

/// Ones words indexed by value (0..13).
/// "eins" is the standalone form; "ein" is used in compounds.
const ONES: [&str; 13] = [
    "null", "eins", "zwei", "drei", "vier", "fuenf", "sechs", "sieben", "acht", "neun", "zehn",
    "elf", "zwoelf",
];

/// Ones words used inside compounds (einundzwanzig, einhundert).
const ONES_COMPOUND: [&str; 10] = [
    "", "ein", "zwei", "drei", "vier", "fuenf", "sechs", "sieben", "acht", "neun",
];

/// Teens 13..19.
const TEENS: [&str; 7] = [
    "dreizehn",
    "vierzehn",
    "fuenfzehn",
    "sechzehn",
    "siebzehn",
    "achtzehn",
    "neunzehn",
];

/// Tens words indexed by tens digit (2..9 → index 0..7).
const TENS: [&str; 8] = [
    "zwanzig",
    "dreissig",
    "vierzig",
    "fuenfzig",
    "sechzig",
    "siebzig",
    "achtzig",
    "neunzig",
];

/// Convert an integer to German words.
///
/// Examples:
/// - `0` → `"null"`
/// - `21` → `"einundzwanzig"`
/// - `200` → `"zweihundert"`
/// - `1000` → `"eintausend"`
/// - `-42` → `"minus zweiundvierzig"`
pub fn number_to_words(n: i64) -> String {
    if n == 0 {
        return "null".to_string();
    }

    if n < 0 {
        let abs_val = (n as u64).wrapping_neg();
        return format!("minus {}", unsigned_to_words(abs_val));
    }

    unsigned_to_words(n as u64)
}

fn unsigned_to_words(n: u64) -> String {
    if n == 0 {
        return "null".to_string();
    }

    let mut parts: Vec<String> = Vec::new();
    let mut remaining = n;

    // German uses long scale: Billion = 10^12, Milliarde = 10^9
    // "eine Million", "zwei Millionen" etc.
    let scales: &[(u64, &str, &str)] = &[
        (1_000_000_000_000, "billion", "billionen"),
        (1_000_000_000, "milliarde", "milliarden"),
        (1_000_000, "million", "millionen"),
        (1_000, "tausend", "tausend"),
    ];

    for &(scale_value, singular, plural) in scales {
        if remaining >= scale_value {
            let chunk = remaining / scale_value;
            remaining %= scale_value;

            if scale_value == 1_000 {
                let thousands_word = if chunk == 1 {
                    "eintausend".to_string()
                } else {
                    format!("{}tausend", chunk_to_words(chunk as u32))
                };

                // In German, remainder < 100 is concatenated directly to thousands:
                // "zweitausendfuenfundzwanzig" (2025) but
                // "eintausend zweihundertvierunddreissig" (1234)
                if remaining > 0 && remaining < 100 {
                    let rest_words = two_digit_to_words(remaining as u32);
                    parts.push(format!("{}{}", thousands_word, rest_words));
                    remaining = 0;
                } else {
                    parts.push(thousands_word);
                }
            } else {
                // Million, Milliarde, etc. are separate words
                let chunk_words = if chunk == 1 {
                    "eine".to_string()
                } else {
                    chunk_to_words(chunk as u32)
                };
                let scale_word = if chunk == 1 { singular } else { plural };
                parts.push(format!("{} {}", chunk_words, scale_word));
            }
        }
    }

    if remaining > 0 {
        parts.push(chunk_to_words(remaining as u32));
    }

    parts.join(" ")
}

/// Convert a number 1..999 to German words.
fn chunk_to_words(n: u32) -> String {
    debug_assert!(n > 0 && n < 1000);
    let hundreds = n / 100;
    let rest = n % 100;

    let mut result = String::new();

    if hundreds > 0 {
        result.push_str(ONES_COMPOUND[hundreds as usize]);
        result.push_str("hundert");
    }

    if rest > 0 {
        result.push_str(&two_digit_to_words(rest));
    }

    result
}

/// Convert 1..99 to German words.
fn two_digit_to_words(n: u32) -> String {
    debug_assert!(n > 0 && n < 100);

    if n <= 12 {
        // Use standalone form for 1 in compound context
        if n == 1 {
            return "eins".to_string();
        }
        return ONES[n as usize].to_string();
    }

    if n < 20 {
        return TEENS[(n - 13) as usize].to_string();
    }

    let tens_idx = (n / 10 - 2) as usize;
    let ones = n % 10;

    if ones == 0 {
        TENS[tens_idx].to_string()
    } else {
        // German: ones-und-tens (reversed order)
        format!("{}und{}", ONES_COMPOUND[ones as usize], TENS[tens_idx])
    }
}

/// Spell each digit of a string individually in German.
pub fn spell_digits(s: &str) -> String {
    s.chars()
        .filter_map(|c| c.to_digit(10).map(|d| ONES[d as usize]))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Convert 1..99 to German words using compound form for 1 (ein instead of eins).
/// Used internally by other modules (e.g., money for "ein euro").
#[allow(dead_code)]
pub(crate) fn two_digit_compound(n: u32) -> String {
    if n == 0 {
        return "null".to_string();
    }
    if n == 1 {
        return "ein".to_string();
    }
    if n <= 12 {
        return ONES[n as usize].to_string();
    }
    if n < 20 {
        return TEENS[(n - 13) as usize].to_string();
    }
    let tens_idx = (n / 10 - 2) as usize;
    let ones = n % 10;
    if ones == 0 {
        TENS[tens_idx].to_string()
    } else {
        format!("{}und{}", ONES_COMPOUND[ones as usize], TENS[tens_idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(number_to_words(0), "null");
        assert_eq!(number_to_words(1), "eins");
        assert_eq!(number_to_words(10), "zehn");
        assert_eq!(number_to_words(11), "elf");
        assert_eq!(number_to_words(12), "zwoelf");
        assert_eq!(number_to_words(13), "dreizehn");
        assert_eq!(number_to_words(17), "siebzehn");
        assert_eq!(number_to_words(20), "zwanzig");
        assert_eq!(number_to_words(21), "einundzwanzig");
        assert_eq!(number_to_words(32), "zweiunddreissig");
    }

    #[test]
    fn test_tens() {
        assert_eq!(number_to_words(30), "dreissig");
        assert_eq!(number_to_words(40), "vierzig");
        assert_eq!(number_to_words(50), "fuenfzig");
        assert_eq!(number_to_words(60), "sechzig");
        assert_eq!(number_to_words(70), "siebzig");
        assert_eq!(number_to_words(80), "achtzig");
        assert_eq!(number_to_words(90), "neunzig");
        assert_eq!(number_to_words(99), "neunundneunzig");
    }

    #[test]
    fn test_hundreds() {
        assert_eq!(number_to_words(100), "einhundert");
        assert_eq!(number_to_words(200), "zweihundert");
        assert_eq!(number_to_words(201), "zweihunderteins");
        assert_eq!(number_to_words(123), "einhundertdreiundzwanzig");
        assert_eq!(number_to_words(999), "neunhundertneunundneunzig");
    }

    #[test]
    fn test_thousands() {
        assert_eq!(number_to_words(1000), "eintausend");
        assert_eq!(number_to_words(2000), "zweitausend");
        assert_eq!(number_to_words(2025), "zweitausendfuenfundzwanzig");
        assert_eq!(
            number_to_words(1234),
            "eintausend zweihundertvierunddreissig"
        );
    }

    #[test]
    fn test_millions() {
        assert_eq!(number_to_words(1000000), "eine million");
        assert_eq!(number_to_words(2000000), "zwei millionen");
        assert_eq!(
            number_to_words(2000003),
            "zwei millionen drei"
        );
    }

    #[test]
    fn test_negative() {
        assert_eq!(number_to_words(-42), "minus zweiundvierzig");
    }

    #[test]
    fn test_spell_digits() {
        assert_eq!(spell_digits("14"), "eins vier");
        assert_eq!(spell_digits("0"), "null");
        assert_eq!(spell_digits("987"), "neun acht sieben");
    }
}
