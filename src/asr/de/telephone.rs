//! Telephone tagger for German.
//!
//! Converts spoken German phone number to written form:
//! - "null vier eins eins eins zwei drei vier eins zwei drei vier" → "(0411) 1234-1234"

use super::cardinal;

/// Parse spoken German telephone number to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trim = input_lower.trim();

    // Convert digit words to digit string
    let tokens: Vec<&str> = input_trim.split_whitespace().collect();
    if tokens.len() < 7 {
        return None;
    }

    let mut digits = String::new();
    for token in &tokens {
        match word_to_digit(token) {
            Some(d) => digits.push_str(d),
            None => return None, // Non-digit word
        }
    }

    // Must have enough digits for a phone number
    if digits.len() < 7 {
        return None;
    }

    // Format as phone number
    // For German phone numbers: area code (first 4 digits) + rest in groups of 4
    if digits.len() == 12 {
        // (XXXX) XXXX-XXXX
        let area = &digits[..4];
        let first = &digits[4..8];
        let second = &digits[8..12];
        return Some(format!("({}) {}-{}", area, first, second));
    }

    // Generic formatting for other lengths
    if digits.len() >= 10 {
        let area = &digits[..4];
        let rest = &digits[4..];
        let mid = rest.len() / 2;
        let first = &rest[..mid];
        let second = &rest[mid..];
        return Some(format!("({}) {}-{}", area, first, second));
    }

    // For shorter numbers, just group
    let mid = digits.len() / 2;
    Some(format!("{}-{}", &digits[..mid], &digits[mid..]))
}

/// Convert German digit word to digit string
fn word_to_digit(word: &str) -> Option<&'static str> {
    match word {
        "null" => Some("0"),
        "eins" | "ein" | "eine" => Some("1"),
        "zwei" => Some("2"),
        "drei" => Some("3"),
        "vier" => Some("4"),
        "fünf" => Some("5"),
        "sechs" => Some("6"),
        "sieben" => Some("7"),
        "acht" => Some("8"),
        "neun" => Some("9"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phone() {
        assert_eq!(
            parse("null vier eins eins eins zwei drei vier eins zwei drei vier"),
            Some("(0411) 1234-1234".to_string())
        );
    }
}
