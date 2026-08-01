//! Decimal number tagger for Korean.
//!
//! Converts spoken Korean decimals to written form:
//! - "삼점일사" → "3.14"
//! - "마이너스 일점영육" → "-1.06"
//! - "영점오" → "0.5"
//!
//! Korean decimals use `점` ("point") between the integer part and the
//! fractional digits. The integer part is a full Sino-Korean number;
//! the fractional part is read digit-by-digit.
//!
//! Known limitation: the fractional-digit scan is greedy. Because
//! Korean digit syllables double as common particles (이 = subject
//! marker, 일 = "day" …), a digit-homograph glued directly onto the
//! fraction with no separator — e.g. `삼점일사이다` ("3.14" + copula
//! 이다) — is over-consumed. ASR output normally inserts a space or a
//! non-Hangul boundary there, which stops the scan correctly.

use super::cardinal;

const MINUS: &str = "마이너스";

/// Process decimal patterns in a string.
pub fn process(input: &str) -> String {
    let mut result = String::new();
    let mut remaining = input;

    while !remaining.is_empty() {
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
    let mut byte_pos = 0;

    for c in input.chars() {
        // 마이너스-prefixed decimal.
        if c == '마' && input[byte_pos..].starts_with(MINUS) {
            let after_minus = &input[byte_pos + MINUS.len()..];
            let after_minus_trimmed = after_minus.trim_start();
            let ws = after_minus.len() - after_minus_trimmed.len();
            if let Some((dec_str, dec_len)) = parse_decimal_at(after_minus_trimmed) {
                let before = &input[..byte_pos];
                let after = &input[byte_pos + MINUS.len() + ws + dec_len..];
                return Some((before, format!("-{}", dec_str), after));
            }
        }

        // Plain decimal starting at a Sino-Korean digit.
        if cardinal::is_sino_korean_numeral(c) {
            if let Some((dec_str, dec_len)) = parse_decimal_at(&input[byte_pos..]) {
                let before = &input[..byte_pos];
                let after = &input[byte_pos + dec_len..];
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
    let jeom_pos = chars.iter().position(|&c| c == '점')?;

    // Integer part: a Sino-Korean number before 점.
    let int_chars = &chars[..jeom_pos];
    if int_chars.is_empty()
        || !int_chars
            .iter()
            .all(|&c| cardinal::is_sino_korean_numeral(c))
    {
        return None;
    }
    let int_val = cardinal::sino_korean_to_number(&int_chars.iter().collect::<String>())?;

    // Fractional part: individual Sino-Korean digits after 점.
    let frac_start = jeom_pos + 1;
    let mut frac_end = frac_start;
    while frac_end < chars.len() && cardinal::sino_korean_digit(chars[frac_end]).is_some() {
        frac_end += 1;
    }
    if frac_end == frac_start {
        return None;
    }

    let frac_digits: String = chars[frac_start..frac_end]
        .iter()
        .map(|&c| cardinal::sino_korean_digit(c).unwrap().to_string())
        .collect();

    let total_bytes: usize = chars[..frac_end].iter().map(|c| c.len_utf8()).sum();
    Some((format!("{}.{}", int_val, frac_digits), total_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(process("삼점일사"), "3.14");
        assert_eq!(process("영점오"), "0.5");
        assert_eq!(process("십점이오"), "10.25");
    }

    #[test]
    fn test_negative() {
        assert_eq!(process("마이너스 일점영육"), "-1.06");
        assert_eq!(process("마이너스일점영육"), "-1.06");
    }

    #[test]
    fn test_contextual() {
        // Non-Hangul / formal-copula boundary stops the fractional scan.
        assert_eq!(process("답은 삼점일사입니다"), "답은 3.14입니다");
        assert_eq!(process("삼점일사 정도"), "3.14 정도");
    }
}
