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
    let original = input.trim();
    let input = original.to_lowercase();
    let input = input.as_str();

    // Handle "zero" specially - NeMo returns "zero" not "0"
    // Preserve original casing: "Zero" → "Zero", "zero" → "zero"
    if input == "zero" {
        return Some(original.to_string());
    }

    // When a single word is capitalized (title-case), preserve standalone
    // small number words that are commonly used as proper nouns/titles
    if original
        .chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or(false)
        && !original.contains(' ')
    {
        match input {
            "twelve" => return Some(original.to_string()),
            _ => {}
        }
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

/// Map a single-digit spoken word to its character form, or `None` if the
/// word isn't a 0-9 digit word. Recognises "oh" / "o" as 0 (common in
/// spelled-out codes and aviation frequencies).
fn single_digit_char(word: &str) -> Option<char> {
    Some(match word {
        "zero" | "oh" | "o" => '0',
        "one" => '1',
        "two" => '2',
        "three" => '3',
        "four" => '4',
        "five" => '5',
        "six" => '6',
        "seven" => '7',
        "eight" => '8',
        "nine" => '9',
        _ => return None,
    })
}

/// Convert spoken number words to integer.
///
/// Two readings are accepted, in order:
/// - **Digit-by-digit** (codes, flight numbers, aviation frequencies):
///   `"one three five"` → `135`. Triggered when every token is a single-digit
///   word (`zero`-`nine`, plus `oh`/`o` for `0`).
/// - **Grammatical** (English number grammar): `"twenty one"` → `21`,
///   `"one hundred twenty three"` → `123`, `"one thousand two hundred thirty
///   four"` → `1234`. Uses a left-to-right accumulator with scale words
///   multiplying the current group.
///
/// Filler words `"and"` and `"a"` are stripped.
///
/// Note: aviation flight-number reading (`"seven eighty eight"` → `788`) is
/// **not** applied here because it conflicts with date and time taggers (e.g.
/// `"twenty one forty two"` must remain readable as old-year `2042` for
/// `date::parse_old_year`). Use [`words_to_number_aviation`] for opt-in
/// flight-number / call-sign contexts.
pub fn words_to_number(input: &str) -> Option<i128> {
    let input = input.to_lowercase();
    let words: Vec<&str> = input
        .split_whitespace()
        .filter(|w| *w != "and" && *w != "a")
        .collect();

    if words.is_empty() {
        return None;
    }

    // Digit-by-digit reading wins whenever it's unambiguous.
    if words.iter().all(|w| single_digit_char(w).is_some()) {
        return words
            .iter()
            .map(|w| single_digit_char(w).unwrap())
            .collect::<String>()
            .parse()
            .ok();
    }

    grammatical_words_to_number(&words)
}

/// Aviation / flight-number / call-sign reading of a number phrase.
///
/// Recognises a leading run of single-digit words concatenated with a trailing
/// grammatical compound, e.g. `"seven eighty eight"` → `788`,
/// `"two thirty five"` → `235`. Falls back to [`words_to_number`] when the
/// aviation pattern does not apply (no digit prefix, scale word present, etc.).
///
/// This is **opt-in**: callers reach for it explicitly from flight-number /
/// call-sign contexts. Generic ITN/TN dispatch keeps using [`words_to_number`]
/// to avoid clobbering date/time/measure semantics (e.g. `"twenty one forty
/// two"` as old-year `2042`).
pub fn words_to_number_aviation(input: &str) -> Option<i128> {
    let input = input.to_lowercase();
    let words: Vec<&str> = input
        .split_whitespace()
        .filter(|w| *w != "and" && *w != "a")
        .collect();

    if words.is_empty() {
        return None;
    }

    // Digit-by-digit reading wins when unambiguous.
    if words.iter().all(|w| single_digit_char(w).is_some()) {
        return words
            .iter()
            .map(|w| single_digit_char(w).unwrap())
            .collect::<String>()
            .parse()
            .ok();
    }

    // Aviation flight-number style: digit prefix + grammatical compound.
    // "seven eighty eight" → "7" ‖ 88 = 788. Skipped if a scale word appears,
    // since "two thousand seventeen" must stay grammatical (= 2017, not 22017).
    let has_scale = words.iter().any(|w| SCALES.contains_key(*w));
    if !has_scale {
        let prefix_len = words
            .iter()
            .take_while(|w| single_digit_char(w).is_some())
            .count();
        if prefix_len >= 1 && prefix_len < words.len() {
            if let Some(rest_num) = grammatical_words_to_number(&words[prefix_len..]) {
                let prefix: String = words[..prefix_len]
                    .iter()
                    .map(|w| single_digit_char(w).unwrap())
                    .collect();
                let combined = format!("{}{}", prefix, rest_num);
                return combined.parse::<i128>().ok();
            }
        }
    }

    grammatical_words_to_number(&words)
}

/// Parse a grammatical English number with running-sum + scale multiplication.
fn grammatical_words_to_number(words: &[&str]) -> Option<i128> {
    // "eleven hundred" = 1100, "twenty hundred" = 2000
    if words.len() == 2 && words[1] == "hundred" {
        if let Some(&val) = ONES.get(words[0]) {
            if (11..=19).contains(&val) {
                return Some((val * 100) as i128);
            }
        }
        if let Some(&val) = TENS.get(words[0]) {
            return Some((val * 100) as i128);
        }
    }

    // "eleven hundred twenty one" = 1100 + 21
    if words.len() >= 2 && words[1] == "hundred" {
        if let Some(&first_val) = ONES.get(words[0]) {
            if (11..=99).contains(&first_val) {
                let base = (first_val * 100) as i128;
                let rest = words[2..].join(" ");
                if let Some(remainder) = words_to_number(&rest) {
                    return Some(base + remainder);
                }
            }
        }
        if let Some(&first_val) = TENS.get(words[0]) {
            let base = (first_val * 100) as i128;
            let rest = words[2..].join(" ");
            if let Some(remainder) = words_to_number(&rest) {
                return Some(base + remainder);
            }
        }
    }

    let mut result: i128 = 0;
    let mut current: i128 = 0;
    let mut found_number = false;

    for &word in words {
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
        assert_eq!(parse("Zero"), Some("Zero".to_string()));
    }

    #[test]
    fn test_twelve_capitalized() {
        assert_eq!(parse("Twelve"), Some("Twelve".to_string()));
        assert_eq!(parse("twelve"), Some("12".to_string()));
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("hello"), None);
        assert_eq!(parse("one hello"), None);
    }

    /// Digit-by-digit reading (issue #15). Sequences of single-digit words
    /// like "one three five" should concatenate to 135, not sum to 9.
    #[test]
    fn test_spelled_digit_sequence() {
        assert_eq!(parse("one three five"), Some("135".to_string()));
        assert_eq!(parse("seven three seven"), Some("737".to_string()));
        assert_eq!(parse("nine one one"), Some("911".to_string()));
        assert_eq!(parse("six two five"), Some("625".to_string()));
        assert_eq!(parse("one two"), Some("12".to_string()));
        // "oh"/"o" read as 0 in spelled codes
        assert_eq!(parse("five oh five"), Some("505".to_string()));
        assert_eq!(parse("four o four"), Some("404".to_string()));
    }

    #[test]
    fn test_words_to_number_digit_sequence() {
        assert_eq!(words_to_number("one three five"), Some(135));
        assert_eq!(words_to_number("six two five"), Some(625));
    }

    /// Aviation flight-number style (issue #14): opt-in helper. A leading run
    /// of single-digit words gets concatenated with the trailing grammatical
    /// compound, e.g. "seven eighty eight" = "7" ‖ 88 = 788. Generic
    /// `words_to_number` deliberately does *not* do this — it would break
    /// `date::parse_old_year` ("twenty one forty two" → 2042) and overlap with
    /// the time tagger ("two thirty five" → 02:35).
    #[test]
    fn test_words_to_number_aviation_flight_number() {
        assert_eq!(words_to_number_aviation("seven eighty eight"), Some(788));
        assert_eq!(words_to_number_aviation("two thirty five"), Some(235));
        assert_eq!(words_to_number_aviation("three forty seven"), Some(347));
        assert_eq!(words_to_number_aviation("nine eleven"), Some(911));
        // Multi-digit prefix.
        assert_eq!(
            words_to_number_aviation("two seven eighty eight"),
            Some(2788)
        );
    }

    /// Aviation helper falls back to grammatical when no digit prefix exists.
    #[test]
    fn test_words_to_number_aviation_falls_back_to_grammatical() {
        assert_eq!(words_to_number_aviation("twenty one"), Some(21));
        assert_eq!(words_to_number_aviation("one hundred"), Some(100));
    }

    /// Aviation helper must keep grammatical reading when a scale word is
    /// present. "two thousand seventeen" must stay 2017, not 22017.
    #[test]
    fn test_words_to_number_aviation_scale_word_forces_grammatical() {
        assert_eq!(
            words_to_number_aviation("two thousand seventeen"),
            Some(2017)
        );
        assert_eq!(
            words_to_number_aviation("two million three"),
            Some(2_000_003)
        );
    }

    /// Generic `words_to_number` (the dispatch path) must NOT do aviation
    /// reading: "seven eighty eight" stays grammatical 95 there, so date/time
    /// taggers see consistent values.
    #[test]
    fn test_words_to_number_no_aviation_reading() {
        assert_eq!(words_to_number("seven eighty eight"), Some(95));
        assert_eq!(words_to_number("twenty one forty two"), Some(63));
        assert_eq!(words_to_number("two thousand seventeen"), Some(2017));
    }
}
