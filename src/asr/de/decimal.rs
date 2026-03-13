//! Decimal number tagger for German.
//!
//! Converts spoken German decimal numbers to written form:
//! - "eins komma zwei millionen" → "1,2 millionen"
//! - "minus sechzig komma zwei vier null null" → "-60,2400"
//! - "acht hundert achtzehn komma drei null drei" → "818,303"

use super::cardinal;

/// Parse spoken German decimal number to written form.
pub fn parse(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    let input_trim = input_lower.trim();

    if !input_trim.contains("komma") {
        // Check for scale-only patterns: "eine million" → "1 million", etc.
        return parse_scale_only(input_trim);
    }

    // Check for negative
    let (is_negative, rest) = if input_trim.starts_with("minus ") {
        (true, input_trim.strip_prefix("minus ")?)
    } else {
        (false, input_trim)
    };

    // Split on "komma"
    let parts: Vec<&str> = rest.splitn(2, "komma").collect();
    if parts.len() != 2 {
        return None;
    }

    let integer_part = parts[0].trim();
    let decimal_rest = parts[1].trim();

    // Parse integer part
    let int_value = if integer_part.is_empty() || integer_part == "null" {
        "0".to_string()
    } else {
        let num = cardinal::words_to_number(integer_part)?;
        num.to_string()
    };

    // Check for scale suffix in decimal part
    let scale_words = ["millionen", "million", "milliarden", "milliarde",
                       "billionen", "billion", "billiarden", "billiarde",
                       "trillionen", "trillion", "tausend"];

    let mut scale_suffix = None;
    let mut decimal_digits_str = decimal_rest.to_string();

    for &sw in &scale_words {
        if decimal_rest.ends_with(sw) {
            let before = decimal_rest[..decimal_rest.len() - sw.len()].trim();
            decimal_digits_str = before.to_string();
            scale_suffix = Some(sw);
            break;
        }
    }

    // Parse decimal digits
    let decimal_digits = parse_decimal_digits(&decimal_digits_str)?;

    let sign = if is_negative { "-" } else { "" };

    if let Some(scale) = scale_suffix {
        Some(format!("{}{},{} {}", sign, int_value, decimal_digits, scale))
    } else {
        Some(format!("{}{},{}", sign, int_value, decimal_digits))
    }
}

/// Parse scale-only patterns: "eine million" → "1 million"
fn parse_scale_only(input: &str) -> Option<String> {
    let scale_patterns = [
        ("millionen", "millionen"),
        ("million", "million"),
        ("milliarden", "milliarden"),
        ("milliarde", "milliarde"),
        ("billionen", "billionen"),
        ("billion", "billion"),
    ];

    for &(spoken, written) in &scale_patterns {
        if input.ends_with(spoken) {
            let num_part = input[..input.len() - spoken.len()].trim();
            if num_part.is_empty() {
                continue;
            }
            let num = cardinal::words_to_number(num_part)?;
            return Some(format!("{} {}", num, written));
        }
    }

    None
}

/// Parse decimal digit words to digit string.
/// "zwei vier null null" → "2400"
/// "drei null drei" → "303"
fn parse_decimal_digits(input: &str) -> Option<String> {
    let digit_map = [
        ("null", "0"), ("eins", "1"), ("ein", "1"),
        ("zwei", "2"), ("drei", "3"), ("vier", "4"),
        ("fünf", "5"), ("sechs", "6"), ("sieben", "7"),
        ("acht", "8"), ("neun", "9"),
    ];

    let tokens: Vec<&str> = input.split_whitespace().collect();
    if tokens.is_empty() {
        return None;
    }

    let mut result = String::new();
    for token in &tokens {
        let mut found = false;
        for &(word, digit) in &digit_map {
            if token == &word {
                result.push_str(digit);
                found = true;
                break;
            }
        }
        if !found {
            return None;
        }
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
    fn test_basic_decimal() {
        assert_eq!(
            parse("acht hundert achtzehn komma drei null drei"),
            Some("818,303".to_string())
        );
    }

    #[test]
    fn test_negative() {
        assert_eq!(
            parse("minus sechzig komma zwei vier null null"),
            Some("-60,2400".to_string())
        );
    }

    #[test]
    fn test_scale() {
        assert_eq!(
            parse("eins komma zwei millionen"),
            Some("1,2 millionen".to_string())
        );
    }

    #[test]
    fn test_scale_only() {
        assert_eq!(
            parse("eine million"),
            Some("1 million".to_string())
        );
    }
}
