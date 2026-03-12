//! Telephone tagger for French.
//!
//! Converts spoken French phone numbers to written form:
//! - "zéro six douze trente-quatre" → "06 12 34"
//! - Handles digit-by-digit or grouped number words

use super::cardinal::words_to_number;

/// Parse spoken French telephone number to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.trim().to_lowercase();

    // Try parsing as a sequence of number words
    if let Some(result) = parse_number_sequence(&input_lower) {
        return Some(result);
    }

    None
}

/// Parse sequence of number words into phone number format
fn parse_number_sequence(input: &str) -> Option<String> {
    let input = input.trim();

    // Split by whitespace and parse each token
    let tokens: Vec<&str> = input.split_whitespace().collect();

    // For phone numbers, expect at least a few tokens
    if tokens.is_empty() {
        return None;
    }

    let mut digits = Vec::new();

    // Try to parse each token/group as a number
    let mut i = 0;
    while i < tokens.len() {
        // Try to parse single token as a digit word (0-9)
        if let Some(num) = parse_single_token(tokens[i]) {
            digits.push(num);
            i += 1;
        } else {
            // Try to parse as number words (e.g., "douze", "vingt et un")
            // For phone numbers, prefer shorter phrases (single words first)
            let mut found = false;
            for len in 1..=std::cmp::min(3, tokens.len() - i) {
                let phrase = tokens[i..i + len].join(" ");
                if let Some(num) = words_to_number(&phrase) {
                    // Convert number to digits string
                    let num_str = (num as i64).to_string();
                    for ch in num_str.chars() {
                        if ch.is_ascii_digit() {
                            digits.push(ch.to_string());
                        }
                    }
                    i += len;
                    found = true;
                    break;
                }
            }
            if !found {
                i += 1;
            }
        }
    }

    // Only return if we got a reasonable number of digits (at least 6 for partial phone numbers)
    if digits.len() >= 6 {
        // Group digits in pairs: "06 12 34 56 78"
        Some(group_phone_digits(&digits))
    } else {
        None
    }
}

/// Parse single token that might be a digit word
fn parse_single_token(token: &str) -> Option<String> {
    let digit_words = [
        ("zéro", "0"),
        ("un", "1"),
        ("deux", "2"),
        ("trois", "3"),
        ("quatre", "4"),
        ("cinq", "5"),
        ("six", "6"),
        ("sept", "7"),
        ("huit", "8"),
        ("neuf", "9"),
    ];

    for (word, digit) in &digit_words {
        if token == *word {
            return Some(digit.to_string());
        }
    }

    None
}

/// Group digits into phone number format: "06 12 34 56 78"
fn group_phone_digits(digits: &[String]) -> String {
    let digit_str: String = digits.iter().map(|s| s.as_str()).collect();

    // Group in pairs
    let mut result = String::new();
    for (i, ch) in digit_str.chars().enumerate() {
        if i > 0 && i % 2 == 0 {
            result.push(' ');
        }
        result.push(ch);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digit_by_digit() {
        assert_eq!(
            parse("zéro six un deux trois quatre"),
            Some("06 12 34".to_string())
        );
    }

    #[test]
    fn test_grouped_numbers() {
        assert_eq!(
            parse("zéro six douze trente-quatre"),
            Some("06 12 34".to_string())
        );
    }

    #[test]
    fn test_full_phone() {
        assert_eq!(
            parse("zéro six douze trente-quatre cinquante-six soixante-dix-huit"),
            Some("06 12 34 56 78".to_string())
        );
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("un deux trois"), None); // Too short
        assert_eq!(parse("hello world"), None);
    }
}
