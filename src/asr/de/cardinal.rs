//! Cardinal number tagger for German.
//!
//! Converts spoken German number words to digits:
//! - "einhundert" → "100"
//! - "einundzwanzig" → "21" (reversed tens)
//! - "minus fünfundzwanzigtausendsiebenunddreißig" → "-25037"
//! - "eine million" → "1000000"

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    /// Single digit and special number words
    static ref ONES: HashMap<&'static str, i64> = {
        let mut m = HashMap::new();
        m.insert("null", 0);
        m.insert("eins", 1);
        m.insert("ein", 1);
        m.insert("eine", 1);
        m.insert("einer", 1);
        m.insert("zwei", 2);
        m.insert("drei", 3);
        m.insert("vier", 4);
        m.insert("fünf", 5);
        m.insert("sechs", 6);
        m.insert("sieben", 7);
        m.insert("acht", 8);
        m.insert("neun", 9);
        m.insert("zehn", 10);
        m.insert("elf", 11);
        m.insert("zwölf", 12);
        m.insert("dreizehn", 13);
        m.insert("vierzehn", 14);
        m.insert("fünfzehn", 15);
        m.insert("sechzehn", 16);
        m.insert("siebzehn", 17);
        m.insert("achtzehn", 18);
        m.insert("neunzehn", 19);
        m
    };

    /// Tens
    static ref TENS: HashMap<&'static str, i64> = {
        let mut m = HashMap::new();
        m.insert("zwanzig", 20);
        m.insert("dreißig", 30);
        m.insert("dreissig", 30);
        m.insert("vierzig", 40);
        m.insert("fünfzig", 50);
        m.insert("sechzig", 60);
        m.insert("siebzig", 70);
        m.insert("achtzig", 80);
        m.insert("neunzig", 90);
        m
    };

    /// Scale words (long scale for German)
    static ref SCALES: HashMap<&'static str, i128> = {
        let mut m = HashMap::new();
        m.insert("hundert", 100);
        m.insert("tausend", 1_000);
        m.insert("million", 1_000_000);
        m.insert("millionen", 1_000_000);
        m.insert("milliarde", 1_000_000_000);
        m.insert("milliarden", 1_000_000_000);
        m.insert("billion", 1_000_000_000_000);
        m.insert("billionen", 1_000_000_000_000);
        m.insert("billiarde", 1_000_000_000_000_000);
        m.insert("billiarden", 1_000_000_000_000_000);
        m.insert("trillion", 1_000_000_000_000_000_000);
        m.insert("trillionen", 1_000_000_000_000_000_000);
        m
    };

    /// Small numbers that pass through as words (0-9)
    static ref PASSTHROUGH: Vec<&'static str> = vec![
        "null", "eins", "ein", "eine", "einer", "zwei", "drei",
        "vier", "fünf", "sechs", "sieben", "acht", "neun",
    ];
}

/// Parse spoken German cardinal number to string representation.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trim = input_lower.trim();

    if input_trim.is_empty() {
        return None;
    }

    // Pass-through single small numbers (0-9)
    if PASSTHROUGH.contains(&input_trim) {
        return Some(input_trim.to_string());
    }

    // Don't parse space-separated sequences of plain digit/ones words
    // without any scale words (hundert, tausend, million, etc.) or "und"
    // This prevents catching phone number digit sequences like
    // "null vier eins eins eins zwei drei vier"
    if input_trim.contains(' ') && !contains_structure_word(input_trim) {
        return None;
    }

    // Check for negative
    let (is_negative, rest) = if input_trim.starts_with("minus ") {
        (true, input_trim.strip_prefix("minus ")?)
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

/// Check if input contains structure words that indicate a compound number
/// (not just a list of digit words)
fn contains_structure_word(input: &str) -> bool {
    let structure_words = [
        "hundert", "tausend", "million", "millionen",
        "milliarde", "milliarden", "billion", "billionen",
        "billiarde", "billiarden", "trillion", "trillionen",
        "und", "minus",
    ];
    let tokens: Vec<&str> = input.split_whitespace().collect();
    tokens.iter().any(|t| {
        structure_words.contains(t) || contains_compound_structure(t)
    })
}

/// Check if a compound word contains scale words
fn contains_compound_structure(word: &str) -> bool {
    let scale_fragments = [
        "hundert", "tausend", "million", "milliard", "billion", "billiard", "trillion",
        "und",
    ];
    // Only check if the word is longer than any known simple word
    if word.len() <= 9 { // "neunzehn" is 8 chars, "sechzehn" is 8
        return false;
    }
    scale_fragments.iter().any(|&f| word.contains(f))
}

/// Convert German number words to a number.
/// Handles both spaced and compound forms.
///
/// Uses a multi-level accumulator:
/// - `result`: flushed value from million+ scale words
/// - `thousands`: value accumulated at the thousands level
/// - `sub`: current ones/tens/hundreds accumulator
pub fn words_to_number(input: &str) -> Option<i128> {
    let normalized = decompose_compound(input);
    let normalized = normalized
        .replace(" und ", " ")
        .replace("  ", " ");

    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    if tokens.is_empty() {
        return None;
    }

    // Check if it's just a single pass-through word
    if tokens.len() == 1 {
        if let Some(&val) = ONES.get(tokens[0]) {
            if val == 0 {
                return None; // "null" should not return 0 from words_to_number
            }
            return Some(val as i128);
        }
        if let Some(&val) = TENS.get(tokens[0]) {
            return Some(val as i128);
        }
        return None;
    }

    let mut result: i128 = 0;     // million+ level
    let mut thousands: i128 = 0;  // thousands level
    let mut sub: i128 = 0;        // ones/tens/hundreds accumulator

    for token in &tokens {
        if let Some(&scale) = SCALES.get(token) {
            if scale == 100 {
                // hundert multiplies sub or assumes 1
                if sub == 0 {
                    sub = 100;
                } else {
                    sub *= 100;
                }
            } else if scale == 1000 {
                // tausend: flush sub into thousands
                if sub == 0 {
                    sub = 1;
                }
                thousands += sub * 1000;
                sub = 0;
            } else {
                // million, milliarde, billion, etc.
                // flush sub + thousands into multiplier for this scale
                let multiplier = thousands + sub;
                let multiplier = if multiplier == 0 { 1 } else { multiplier };
                result += multiplier * scale;
                thousands = 0;
                sub = 0;
            }
        } else if let Some(&val) = ONES.get(token) {
            sub += val as i128;
        } else if let Some(&val) = TENS.get(token) {
            sub += val as i128;
        } else {
            return None; // Unknown word
        }
    }

    result += thousands + sub;

    if result == 0 {
        None
    } else {
        Some(result)
    }
}

/// Public wrapper for decompose_compound, used by date parser for year patterns.
pub fn decompose_compound_public(input: &str) -> String {
    decompose_compound(input)
}

/// Decompose German compound number words into space-separated tokens.
///
/// E.g., "einhundertzwei" → "ein hundert zwei"
///       "fünfundzwanzigtausendsiebenunddreißig" → "fünf und zwanzig tausend sieben und dreißig"
fn decompose_compound(input: &str) -> String {
    // First, normalize the input by replacing hyphens with spaces
    let input = input.replace('-', " ");

    // Process each space-separated token
    let tokens: Vec<&str> = input.split_whitespace().collect();
    let mut result_parts: Vec<String> = Vec::new();

    for token in tokens {
        // Check if the token is already a known word
        if is_known_word(token) {
            result_parts.push(token.to_string());
            continue;
        }

        // Try to decompose the compound word
        if let Some(decomposed) = decompose_single_compound(token) {
            result_parts.push(decomposed);
        } else {
            result_parts.push(token.to_string());
        }
    }

    result_parts.join(" ")
}

/// Check if a token is a known number word
fn is_known_word(token: &str) -> bool {
    ONES.contains_key(token) || TENS.contains_key(token) || SCALES.contains_key(token)
        || token == "und" || token == "minus"
}

/// Decompose a single compound German number word.
fn decompose_single_compound(word: &str) -> Option<String> {
    let mut remaining = word.to_string();
    let mut parts: Vec<String> = Vec::new();

    while !remaining.is_empty() {
        let mut found = false;

        // Try scale words first (longest match)
        let scale_words = [
            "trillionen", "trillion",
            "billiarden", "billiarde",
            "billionen", "billion",
            "milliarden", "milliarde",
            "millionen", "million",
            "tausend",
            "hundert",
        ];

        for &sw in &scale_words {
            if remaining.starts_with(sw) {
                parts.push(sw.to_string());
                remaining = remaining[sw.len()..].to_string();
                found = true;
                break;
            }
        }
        if found { continue; }

        // Try "und" connector
        if remaining.starts_with("und") {
            parts.push("und".to_string());
            remaining = remaining[3..].to_string();
            continue;
        }

        // Try teens and special words (longest first)
        let teen_words = [
            "neunzehn", "achtzehn", "siebzehn", "sechzehn",
            "fünfzehn", "vierzehn", "dreizehn", "zwölf", "elf",
        ];
        for &tw in &teen_words {
            if remaining.starts_with(tw) {
                parts.push(tw.to_string());
                remaining = remaining[tw.len()..].to_string();
                found = true;
                break;
            }
        }
        if found { continue; }

        // Try tens (longest first)
        let tens_words = [
            "neunzig", "achtzig", "siebzig", "sechzig",
            "fünfzig", "vierzig", "dreißig", "dreissig", "zwanzig",
        ];
        for &tw in &tens_words {
            if remaining.starts_with(tw) {
                parts.push(tw.to_string());
                remaining = remaining[tw.len()..].to_string();
                found = true;
                break;
            }
        }
        if found { continue; }

        // Try ones (check longer words first to avoid partial matches)
        let ones_words = [
            "sieben", "einer", "eine", "eins", "ein",
            "neun", "acht", "fünf", "vier", "drei", "zwei",
            "sechs", "zehn", "null",
        ];
        for &ow in &ones_words {
            if remaining.starts_with(ow) {
                parts.push(ow.to_string());
                remaining = remaining[ow.len()..].to_string();
                found = true;
                break;
            }
        }
        if found { continue; }

        // Unknown character sequence - not a valid compound number
        return None;
    }

    if parts.len() > 1 {
        Some(parts.join(" "))
    } else if parts.len() == 1 {
        Some(parts[0].clone())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passthrough() {
        assert_eq!(parse("null"), Some("null".to_string()));
        assert_eq!(parse("eins"), Some("eins".to_string()));
        assert_eq!(parse("ein"), Some("ein".to_string()));
        assert_eq!(parse("eine"), Some("eine".to_string()));
        assert_eq!(parse("einer"), Some("einer".to_string()));
        assert_eq!(parse("zwei"), Some("zwei".to_string()));
        assert_eq!(parse("neun"), Some("neun".to_string()));
    }

    #[test]
    fn test_teens() {
        assert_eq!(parse("zehn"), Some("10".to_string()));
        assert_eq!(parse("elf"), Some("11".to_string()));
        assert_eq!(parse("zwölf"), Some("12".to_string()));
        assert_eq!(parse("achtzehn"), Some("18".to_string()));
    }

    #[test]
    fn test_tens() {
        assert_eq!(parse("zwanzig"), Some("20".to_string()));
        assert_eq!(parse("dreißig"), Some("30".to_string()));
        assert_eq!(parse("neunzig"), Some("90".to_string()));
    }

    #[test]
    fn test_hundreds() {
        assert_eq!(parse("einhundert"), Some("100".to_string()));
        assert_eq!(parse("ein hundert"), Some("100".to_string()));
        assert_eq!(parse("einhundertzwei"), Some("102".to_string()));
    }

    #[test]
    fn test_compound() {
        assert_eq!(parse("einundzwanzig"), Some("21".to_string()));
        assert_eq!(parse("eintausend"), Some("1000".to_string()));
        assert_eq!(parse("eintausendzwanzig"), Some("1020".to_string()));
    }

    #[test]
    fn test_negative() {
        assert_eq!(parse("minus sechzig"), Some("-60".to_string()));
    }

    #[test]
    fn test_large() {
        assert_eq!(parse("eine million"), Some("1000000".to_string()));
        assert_eq!(parse("zwei millionen drei"), Some("2000003".to_string()));
    }
}
