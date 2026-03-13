//! Decimal number tagger for French.
//!
//! Converts spoken French decimal numbers to written form:
//! - "trois virgule un quatre" → "3,14"
//! - "zero virgule cinq" → "0,5"
//! - "huit cent dix-huit virgule trois zéro trois" → "818,303"
//! - "mille-huit-cent-dix-huit virgule trois zéro trois trois quatre" → "1 818,303 34"

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
            if num_part.contains(" virgule ") || num_part.contains("virgule ") {
                let decimal = parse_virgule_decimal(num_part)?;
                return Some(format!("{} {}", decimal, orig_scale));
            }

            // Plain number with scale
            let num = parse_integer_part(num_part)?;
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

    // Integer part
    let integer_part = if integer_str.is_empty() {
        String::new()
    } else {
        let n = parse_integer_part(integer_str)?;
        format_with_spaces(n)
    };

    // Decimal part - parse as individual digits, with compound number support
    let decimal_raw = parse_decimal_digits(decimal_str)?;

    // Format decimal part with space separators (groups of 3 from left)
    let decimal_part = format_decimal_with_spaces(&decimal_raw);

    let sign = if is_negative { "-" } else { "" };

    if integer_part.is_empty() {
        Some(format!("{},{}", sign, decimal_part))
    } else {
        Some(format!("{}{},{}", sign, integer_part, decimal_part))
    }
}

/// Parse integer part from words, handling both space-separated and hyphenated forms
fn parse_integer_part(input: &str) -> Option<i64> {
    let normalized = input.trim();
    if normalized.is_empty() {
        return None;
    }

    // Handle "zéro"/"zero"
    let lower = normalized.to_lowercase();
    if lower == "zéro" || lower == "zero" {
        return Some(0);
    }

    words_to_number(&lower).map(|n| n as i64)
}

/// Format number with French space separators for thousands
fn format_with_spaces(n: i64) -> String {
    let abs_n = n.unsigned_abs();
    let s = abs_n.to_string();

    if s.len() <= 3 {
        return if n < 0 {
            format!("-{}", s)
        } else {
            s
        };
    }

    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    for (i, &c) in chars.iter().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(' ');
        }
        result.push(c);
    }

    if n < 0 {
        format!("-{}", result)
    } else {
        result
    }
}

/// Format decimal digits with space separators (groups of 3 from left)
/// "2400" → "240 0", "303" → "303", "30334" → "303 34"
fn format_decimal_with_spaces(digits: &str) -> String {
    if digits.len() <= 3 {
        return digits.to_string();
    }

    let mut result = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(' ');
        }
        result.push(c);
    }
    result
}

/// Parse decimal digits: "un quatre" → "14", "zéro cinq" → "05"
/// Each word is independently converted to its digit value:
/// - "trente" → "30", "trois" → "3", so "trente trois" → "303"
/// - "vingt-huit" → "28" (hyphenated compound = single token)
fn parse_decimal_digits(input: &str) -> Option<String> {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    let mut result = String::new();

    for token in tokens {
        // Try single digit word first
        if let Some(digit) = digit_word_to_char(token) {
            result.push(digit);
            continue;
        }

        // Try as a compound number (single token, possibly hyphenated)
        if let Some(num) = words_to_number(token) {
            let num = num as i64;
            if num >= 0 {
                for c in num.to_string().chars() {
                    result.push(c);
                }
                continue;
            }
        }

        return None;
    }

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

/// Convert single digit word to char
fn digit_word_to_char(word: &str) -> Option<char> {
    match word {
        "zéro" | "zero" => Some('0'),
        "un" | "une" => Some('1'),
        "deux" => Some('2'),
        "trois" => Some('3'),
        "quatre" => Some('4'),
        "cinq" => Some('5'),
        "six" => Some('6'),
        "sept" => Some('7'),
        "huit" => Some('8'),
        "neuf" => Some('9'),
        _ => None,
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
            parse("moins soixante virgule deux quatre zéro zéro"),
            Some("-60,240 0".to_string())
        );
    }

    #[test]
    fn test_compound_integer() {
        assert_eq!(
            parse("huit cent dix-huit virgule trois zéro trois"),
            Some("818,303".to_string())
        );
        assert_eq!(
            parse("huit-cent-dix-huit virgule trois zéro trois"),
            Some("818,303".to_string())
        );
    }

    #[test]
    fn test_large_with_spaces() {
        assert_eq!(
            parse("mille-huit-cent-dix-huit virgule trois zéro trois trois quatre"),
            Some("1 818,303 34".to_string())
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
            parse("zéro virgule deux million"),
            Some("0,2 million".to_string())
        );
    }
}
