//! Decimal number tagger for Chinese.
//!
//! Converts Chinese decimal expressions to Arabic numeral form:
//! - "一点零五六" → "1.056"
//! - "负五万点二四五" → "-50,000.245"
//! - "壹佰点叁肆" → "100.34"
//!
//! Handles: 点/點 as decimal point, 负/負 as negative prefix,
//! traditional/financial characters.

use super::cardinal;

/// Process decimal patterns in a string.
pub fn process(input: &str) -> String {
    let mut result = String::new();
    let mut remaining = input;

    while !remaining.is_empty() {
        if let Some((before, dec_str, after)) = find_decimal(remaining) {
            result.push_str(before);
            result.push_str(&dec_str);
            remaining = after;
        } else {
            result.push_str(remaining);
            break;
        }
    }

    result
}

/// Find the next decimal expression in the string.
fn find_decimal(input: &str) -> Option<(&str, String, &str)> {
    let chars: Vec<char> = input.chars().collect();
    let mut byte_pos = 0;

    for (_i, &c) in chars.iter().enumerate() {
        // Check for 负/負 prefix
        if c == '负' || c == '負' {
            let after_neg = &input[byte_pos + c.len_utf8()..];
            if let Some((dec_str, dec_byte_len)) = parse_decimal_at(after_neg) {
                let before = &input[..byte_pos];
                let after = &input[byte_pos + c.len_utf8() + dec_byte_len..];
                return Some((before, format!("-{}", dec_str), after));
            }
        }

        // Check for Chinese digit that could start a decimal
        if cardinal::is_zh_numeral(c) {
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

    // Find 点/點 position
    let dian_pos = chars.iter().position(|&c| c == '点' || c == '點')?;

    // Integer part: Chinese numerals before 点
    let int_chars: Vec<char> = chars[..dian_pos].to_vec();
    if int_chars.is_empty() {
        return None;
    }

    // All int chars must be Chinese numerals
    if !int_chars.iter().all(|&c| cardinal::is_zh_numeral(c)) {
        return None;
    }

    // Parse integer part — fully expand (no 万-preservation for decimals)
    let int_str: String = int_chars.iter().collect();
    let int_val = cardinal::zh_to_number(&int_str)?;
    let int_formatted = cardinal::format_with_commas(int_val);

    // Fractional part: individual Chinese digits after 点
    let frac_start = dian_pos + 1;
    let mut frac_end = frac_start;
    while frac_end < chars.len() {
        let c = chars[frac_end];
        if cardinal::zh_digit(c).is_some() {
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
        .map(|&c| cardinal::zh_digit(c).unwrap().to_string())
        .collect();

    let total_bytes: usize = chars[..frac_end].iter().map(|c| c.len_utf8()).sum();

    Some((format!("{}.{}", int_formatted, frac_digits), total_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(process("一点零五六"), "1.056");
        assert_eq!(process("两百点一"), "200.1");
    }

    #[test]
    fn test_negative() {
        assert_eq!(process("负五万点二四五"), "-50,000.245");
        assert_eq!(process("负一点一"), "-1.1");
    }

    #[test]
    fn test_traditional() {
        assert_eq!(process("一點零零五"), "1.005");
        assert_eq!(process("負十點五"), "-10.5");
    }

    #[test]
    fn test_financial() {
        assert_eq!(process("壹佰点叁肆"), "100.34");
        assert_eq!(process("伍拾壹点肆"), "51.4");
    }
}
