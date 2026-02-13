//! Decimal number tagger.
//!
//! Converts spoken decimal numbers to written form:
//! - "three point one four" → "3.14"
//! - "zero point five" → "0.5"
//! - "five point two million" → "5.2 million"
//! - "point five" → ".5"

use super::cardinal::words_to_number;

/// Parse spoken decimal expression to written form.
pub fn parse(input: &str) -> Option<String> {
    let original = input.trim();
    let input_lower = original.to_lowercase();

    // Check for scale suffix (million, billion, etc.)
    if let Some(result) = parse_with_scale(original, &input_lower) {
        return Some(result);
    }

    // Check for "point" decimal
    if let Some(result) = parse_point_decimal(&input_lower) {
        return Some(result);
    }

    None
}

/// Parse numbers with scale words (million, billion, trillion)
fn parse_with_scale(original: &str, input_lower: &str) -> Option<String> {
    let scales = ["trillion", "billion", "million", "thousand"];

    for scale in &scales {
        if input_lower.ends_with(scale) {
            let num_part = input_lower[..input_lower.len() - scale.len()].trim();

            // Extract original scale word to preserve casing
            let orig_scale = &original[original.len() - scale.len()..];

            // Check if it has a decimal point
            if num_part.contains(" point ") {
                let decimal = parse_point_decimal(num_part)?;
                return Some(format!("{} {}", decimal, orig_scale));
            }

            // Plain number with scale
            let num = words_to_number(num_part)? as i64;
            return Some(format!("{} {}", num, orig_scale));
        }
    }

    None
}

/// Parse "X point Y" decimal pattern
fn parse_point_decimal(input: &str) -> Option<String> {
    // Handle negative
    let (is_negative, rest) = if input.starts_with("minus ") {
        (true, input.strip_prefix("minus ")?)
    } else if input.starts_with("negative ") {
        (true, input.strip_prefix("negative ")?)
    } else {
        (false, input)
    };

    // Handle "point X" (no integer part, e.g., "point five" → ".5")
    let (integer_str, decimal_str) = if rest.starts_with("point ") {
        ("", rest.strip_prefix("point ")?)
    } else if rest.contains(" point ") {
        let parts: Vec<&str> = rest.splitn(2, " point ").collect();
        if parts.len() != 2 {
            return None;
        }
        (parts[0], parts[1])
    } else {
        return None;
    };

    // Integer part (can be empty for ".5")
    let integer_part = if integer_str.is_empty() {
        String::new()
    } else {
        (words_to_number(integer_str)? as i64).to_string()
    };

    // Decimal part - parse as individual digits
    let decimal_part = parse_decimal_digits(decimal_str)?;

    let sign = if is_negative { "-" } else { "" };

    if integer_part.is_empty() {
        Some(format!("{}.{}", sign, decimal_part))
    } else {
        Some(format!("{}{}.{}", sign, integer_part, decimal_part))
    }
}

/// Parse decimal digits: "one four" → "14", "o five" → "05"
fn parse_decimal_digits(input: &str) -> Option<String> {
    let words: Vec<&str> = input.split_whitespace().collect();
    let mut result = String::new();

    for word in words {
        let digit = match word {
            "zero" | "o" | "oh" => '0',
            "one" => '1',
            "two" => '2',
            "three" => '3',
            "four" => '4',
            "five" => '5',
            "six" => '6',
            "seven" => '7',
            "eight" => '8',
            "nine" => '9',
            // Handle compound numbers like "twenty six" → "26"
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
        assert_eq!(parse("three point one four"), Some("3.14".to_string()));
        assert_eq!(parse("zero point five"), Some("0.5".to_string()));
        assert_eq!(parse("zero point two six"), Some("0.26".to_string()));
    }

    #[test]
    fn test_point_only() {
        assert_eq!(parse("point five"), Some(".5".to_string()));
        assert_eq!(parse("point zero two"), Some(".02".to_string()));
    }

    #[test]
    fn test_with_oh() {
        assert_eq!(parse("eighteen point o five"), Some("18.05".to_string()));
        assert_eq!(parse("eighteen point o o o"), Some("18.000".to_string()));
    }

    #[test]
    fn test_negative() {
        assert_eq!(parse("minus sixty point two four zero zero"), Some("-60.2400".to_string()));
    }

    #[test]
    fn test_with_scale() {
        assert_eq!(parse("five point two million"), Some("5.2 million".to_string()));
        assert_eq!(parse("fifty billion"), Some("50 billion".to_string()));
        assert_eq!(parse("four point eight five billion"), Some("4.85 billion".to_string()));
    }
}
