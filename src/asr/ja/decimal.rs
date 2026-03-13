//! Decimal number tagger for Japanese.
//!
//! Converts spoken Japanese decimals to written form:
//! - "マイナス一点零六" → "-1.06"
//! - "五点三" → "5.3"

use super::cardinal;

/// Process decimal patterns in a string.
/// Handles: マイナスX点YZ → -X.YZ
pub fn process(input: &str) -> String {
    let mut result = String::new();
    let mut remaining = input;

    while !remaining.is_empty() {
        // Try to find マイナス or a kanji number followed by 点
        if let Some((before, decimal_str, after)) = find_decimal(remaining) {
            result.push_str(before);
            result.push_str(&decimal_str);
            remaining = after;
        } else {
            result.push_str(remaining);
            break;
        }
    }

    result
}

/// Find the next decimal expression in the string.
/// Returns (before, converted_decimal, after).
fn find_decimal(input: &str) -> Option<(&str, String, &str)> {
    // Look for マイナス followed by decimal, or plain decimal (X点Y)
    let chars: Vec<char> = input.chars().collect();
    let mut byte_pos = 0;

    for (i, &c) in chars.iter().enumerate() {
        // Check for マイナス prefix
        if c == 'マ' && input[byte_pos..].starts_with("マイナス") {
            let minus_len = "マイナス".len();
            let after_minus = &input[byte_pos + minus_len..];
            if let Some((dec_str, dec_byte_len)) = parse_decimal_at(after_minus) {
                let before = &input[..byte_pos];
                let after = &input[byte_pos + minus_len + dec_byte_len..];
                return Some((before, format!("-{}", dec_str), after));
            }
        }

        // Check for kanji digit that could start a decimal
        if cardinal::is_kanji_numeral(c) || c == '零' {
            if let Some((dec_str, dec_byte_len)) = parse_decimal_at(&input[byte_pos..]) {
                let before = &input[..byte_pos];
                let after = &input[byte_pos + dec_byte_len..];
                return Some((before, dec_str, after));
            }
        }

        byte_pos += c.len_utf8();
    }

    None
}

/// Try to parse a decimal number starting at the given position.
/// Returns (formatted_string, bytes_consumed).
fn parse_decimal_at(input: &str) -> Option<(String, usize)> {
    let chars: Vec<char> = input.chars().collect();
    if chars.is_empty() {
        return None;
    }

    // Find 点 position
    let ten_pos = chars.iter().position(|&c| c == '点')?;

    // Integer part: kanji before 点
    let int_chars: Vec<char> = chars[..ten_pos].to_vec();
    if int_chars.is_empty() {
        return None;
    }

    // All int chars must be kanji numerals
    if !int_chars.iter().all(|&c| cardinal::is_kanji_numeral(c)) {
        return None;
    }

    let int_val = cardinal::kanji_to_number(&int_chars.iter().collect::<String>())?;

    // Fractional part: individual kanji digits after 点
    let frac_start = ten_pos + 1;
    let mut frac_end = frac_start;
    while frac_end < chars.len() {
        let c = chars[frac_end];
        if cardinal::kanji_digit(c).is_some() {
            frac_end += 1;
        } else {
            break;
        }
    }

    if frac_end == frac_start {
        return None; // No fractional digits
    }

    let frac_digits: String = chars[frac_start..frac_end]
        .iter()
        .map(|&c| cardinal::kanji_digit(c).unwrap().to_string())
        .collect();

    let total_bytes: usize = chars[..frac_end].iter().map(|c| c.len_utf8()).sum();

    Some((format!("{}.{}", int_val, frac_digits), total_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(process("マイナス一点零六"), "-1.06");
        assert_eq!(process("五点三"), "5.3");
    }

    #[test]
    fn test_contextual() {
        assert_eq!(process("答えはマイナス一点零六"), "答えは-1.06");
    }
}
