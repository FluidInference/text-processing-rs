//! Telephone tagger for French.
//!
//! Converts spoken French phone numbers to written form:
//! - "zéro six douze trente-quatre" → "06 12 34"
//! - "double neuf douze trente-deux" → "99 12 32"
//! - Handles digit-by-digit or grouped number words
//!
//! French phone numbers are formatted as 2-digit groups: "02 12 32 30 30"
//! Standard French numbers are 10 digits; if 9 digits are provided,
//! a leading zero is prepended (implied area code).

use super::cardinal::words_to_number;

/// Parse spoken French telephone number to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.trim().to_lowercase();

    parse_number_sequence(&input_lower)
}

/// Parse sequence of number words into phone number format.
fn parse_number_sequence(input: &str) -> Option<String> {
    let input = input.trim();

    let tokens: Vec<&str> = input.split_whitespace().collect();
    if tokens.is_empty() {
        return None;
    }

    let mut digits = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        // Handle "double X" → XX
        if tokens[i] == "double" && i + 1 < tokens.len() {
            if let Some(d) = parse_single_digit(tokens[i + 1]) {
                digits.push(d);
                digits.push(d);
                i += 2;
                continue;
            }
        }

        // Handle "triple X" → XXX
        if tokens[i] == "triple" && i + 1 < tokens.len() {
            if let Some(d) = parse_single_digit(tokens[i + 1]) {
                digits.push(d);
                digits.push(d);
                digits.push(d);
                i += 2;
                continue;
            }
        }

        // Try to parse single digit word (zéro-neuf)
        if let Some(d) = parse_single_digit(tokens[i]) {
            digits.push(d);
            i += 1;
            continue;
        }

        // Try single-token compound number: "douze" → 12, "trente-deux" → 32
        // Only parse single tokens to avoid greedily combining separate groups
        if let Some(num) = words_to_number(tokens[i]) {
            let num = num as u32;
            if num >= 10 && num <= 99 {
                digits.push((num / 10) as u8);
                digits.push((num % 10) as u8);
            } else if num < 10 {
                digits.push(num as u8);
            } else {
                return None;
            }
            i += 1;
        } else {
            return None;
        }
    }

    // Need at least 6 digits for a phone number
    if digits.len() < 6 {
        return None;
    }

    // French phone numbers are 10 digits; if 9 provided, prepend 0
    if digits.len() == 9 {
        digits.insert(0, 0);
    }

    // Format as 2-digit groups: "02 12 32 30 30"
    let mut result = String::new();
    for (idx, &d) in digits.iter().enumerate() {
        if idx > 0 && idx % 2 == 0 {
            result.push(' ');
        }
        result.push(char::from(b'0' + d));
    }

    Some(result)
}

/// Parse single digit word (0-9), including "une"
fn parse_single_digit(token: &str) -> Option<u8> {
    match token {
        "zéro" | "zero" => Some(0),
        "un" | "une" => Some(1),
        "deux" => Some(2),
        "trois" => Some(3),
        "quatre" => Some(4),
        "cinq" => Some(5),
        "six" => Some(6),
        "sept" => Some(7),
        "huit" => Some(8),
        "neuf" => Some(9),
        _ => None,
    }
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
    fn test_without_leading_zero() {
        assert_eq!(
            parse("deux douze trente-deux trente trente"),
            Some("02 12 32 30 30".to_string())
        );
    }

    #[test]
    fn test_digit_by_digit_with_une() {
        assert_eq!(
            parse("deux une deux trois deux trois zéro trois zéro"),
            Some("02 12 32 30 30".to_string())
        );
    }

    #[test]
    fn test_double() {
        assert_eq!(
            parse("double neuf douze trente-deux trente trente"),
            Some("99 12 32 30 30".to_string())
        );
    }

    #[test]
    fn test_invalid() {
        assert_eq!(parse("un deux trois"), None); // Too short
        assert_eq!(parse("hello world"), None);
    }
}
