//! Decimal number tagger for French.
//!
//! Converts spoken French decimal numbers to written form:
//! - "trois virgule un quatre" → "3,14"
//! - "zero virgule cinq" → "0,5"
//! - "cinq virgule deux millions" → "5,2 millions"

use super::cardinal::words_to_number;

/// Parse spoken French decimal expression to written form.
pub fn parse(input: &str) -> Option<String> {
    let original = input.trim();
    let input_lower = original.to_lowercase();

    // Check for scale suffix (million, milliard, etc.)
    if let Some(result) = parse_with_scale(original, &input_lower) {
        return Some(result);
    }

    // Check for "virgule" decimal
    if let Some(result) = parse_virgule_decimal(&input_lower) {
        return Some(result);
    }

    None
}

/// Parse numbers with scale words (million, milliard, billion, etc.)
fn parse_with_scale(original: &str, input_lower: &str) -> Option<String> {
    let scales = [
        "trillions",
        "trillion",
        "billiards",
        "billiard",
        "billions",
        "billion",
        "milliards",
        "milliard",
        "millions",
        "million",
        "mille",
    ];

    for scale in &scales {
        if input_lower.ends_with(scale) {
            let num_part = input_lower[..input_lower.len() - scale.len()].trim();

            // Extract original scale word to preserve casing
            let orig_scale = &original[original.len() - scale.len()..];

            // Check if it has a decimal point
            if num_part.contains(" virgule ") {
                let decimal = parse_virgule_decimal(num_part)?;
                return Some(format!("{} {}", decimal, orig_scale));
            }

            // Plain number with scale
            let num = words_to_number(num_part)? as i64;
            return Some(format!("{} {}", num, orig_scale));
        }
    }

    None
}

/// Parse "X virgule Y" decimal pattern
fn parse_virgule_decimal(input: &str) -> Option<String> {
    // Handle negative
    let (is_negative, rest) = if input.starts_with("moins ") {
        (true, input.strip_prefix("moins ")?)
    } else {
        (false, input)
    };

    // Handle "virgule X" (no integer part, e.g., "virgule cinq" → ",5")
    let (integer_str, decimal_str) = if rest.starts_with("virgule ") {
        ("", rest.strip_prefix("virgule ")?)
    } else if rest.contains(" virgule ") {
        let parts: Vec<&str> = rest.splitn(2, " virgule ").collect();
        if parts.len() != 2 {
            return None;
        }
        (parts[0], parts[1])
    } else {
        return None;
    };

    // Integer part (can be empty for ",5")
    let integer_part = if integer_str.is_empty() {
        String::new()
    } else if integer_str == "zero" {
        "0".to_string()
    } else {
        (words_to_number(integer_str)? as i64).to_string()
    };

    // Decimal part - parse as individual digits
    let decimal_part = parse_decimal_digits(decimal_str)?;

    let sign = if is_negative { "-" } else { "" };

    if integer_part.is_empty() {
        Some(format!("{},{}", sign, decimal_part))
    } else {
        Some(format!("{}{},{}", sign, integer_part, decimal_part))
    }
}

/// Parse decimal digits: "un quatre" → "14", "zero cinq" → "05"
fn parse_decimal_digits(input: &str) -> Option<String> {
    let words: Vec<&str> = input.split_whitespace().collect();
    let mut result = String::new();

    for word in words {
        let digit = match word {
            "zero" => '0',
            "un" | "une" => '1',
            "deux" => '2',
            "trois" => '3',
            "quatre" => '4',
            "cinq" => '5',
            "six" => '6',
            "sept" => '7',
            "huit" => '8',
            "neuf" => '9',
            // Handle compound numbers
            _ => {
                // Try to parse as a number
                if let Some(num) = words_to_number(word) {
                    for c in (num as i64).to_string().chars() {
                        result.push(c);
                    }
                    continue;
                }
                return None;
            }
        };
        result.push(digit);
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_decimal() {
        assert_eq!(parse("trois virgule un quatre"), Some("3,14".to_string()));
        assert_eq!(parse("zero virgule cinq"), Some("0,5".to_string()));
        assert_eq!(parse("zero virgule deux six"), Some("0,26".to_string()));
    }

    #[test]
    fn test_virgule_only() {
        assert_eq!(parse("virgule cinq"), Some(",5".to_string()));
        assert_eq!(parse("virgule zero deux"), Some(",02".to_string()));
    }

    #[test]
    fn test_negative() {
        assert_eq!(
            parse("moins soixante virgule deux quatre zero zero"),
            Some("-60,2400".to_string())
        );
    }

    #[test]
    fn test_with_scale() {
        assert_eq!(
            parse("cinq virgule deux millions"),
            Some("5,2 millions".to_string())
        );
        assert_eq!(
            parse("cinquante milliards"),
            Some("50 milliards".to_string())
        );
        assert_eq!(
            parse("quatre virgule huit cinq milliards"),
            Some("4,85 milliards".to_string())
        );
    }
}
